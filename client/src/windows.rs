use crate::{fs, net, windows, Result};
use std::{
    ffi::{OsStr, OsString},
    os::windows::ffi::OsStrExt,
    ptr,
};
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;
use winapi::{
    ctypes::c_int,
    shared::minwindef::DWORD,
    um::{
        processthreadsapi::GetCurrentProcessId,
        shellapi::ShellExecuteW,
        wincon::GetConsoleWindow,
        winuser::{GetWindowThreadProcessId, SW_SHOW},
    },
};

// TODO: We should remove the installer after successful update!
// TODO: Directly download from github!

/// Tries to self update incase a newer version got released.
pub(crate) async fn update() -> Result<()> {
    // Note: this will ignore network errors silently.
    if let Some(url) = net::check_win_update().await.ok().flatten() {
        log::info!(
            "Found airshipper update! It's highly recommended to update. Install? [Y/n]"
        );
        if crate::cli::confirm_action()? {
            let mut resp = net::query(&url).await?;
            let path = fs::get_cache_path();

            let filename = match Url::parse(&url)?
                .path_segments()
                .map(|x| x.last())
                .flatten()
            {
                Some(name) => name.to_string(),
                None => {
                    return Err(
                        format!("Malformed update url for airshipper! {}", url).into()
                    );
                },
            };

            if resp.status().is_success() {
                log::debug!(
                    "Download airshipper update to: {}",
                    path.join(&filename).display()
                );

                let mut file = File::create(&path.join(&filename)).await?;
                while let Some(chunk) = resp.chunk().await? {
                    file.write_all(&chunk).await?;
                }
                file.sync_all().await?;

                // Free up access to file.
                drop(file);

                // Execute msi installer
                let result = windows::execute_as_admin(
                    "msiexec",
                    &format!(
                        "/passive /i {} /L*V {} AUTOSTART=1",
                        path.join(&filename).display(),
                        path.join("airshipper-install.log").display()
                    ),
                );

                if result <= 32 {
                    log::error!(
                        "Failed to update airshipper! {}",
                        std::io::Error::last_os_error()
                    );
                }
                std::process::exit(0);
            }
        }
    } else {
        log::debug!("Airshipper is up-to-date!");
    }
    Ok(())
}

pub fn execute_as_admin<T>(program: T, args: T) -> i32
where
    T: Into<OsString>,
{
    let operation: Vec<u16> = OsStr::new("runas\0").encode_wide().collect();
    let mut program = program.into();
    program.push("\0");
    let mut arguments = args.into();
    arguments.push("\0");

    let bin = program.encode_wide().collect::<Vec<u16>>();
    let arguments: Vec<u16> = arguments.encode_wide().collect();

    unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            bin.as_ptr(),
            arguments.as_ptr(),
            ptr::null(),
            SW_SHOW,
        ) as c_int
    }
}

/// Detaches the console incase the process hasn't been started from one.
///
/// Will exit incase of an error.
pub fn detach_non_inherited_console() {
    if !started_from_console() {
        let code = unsafe { winapi::um::wincon::FreeConsole() };
        if code == 0 {
            eprintln!("Unable to detach the console!");
            std::process::exit(1);
        }
    }
}

/// Determines whether the process has been started from console.
fn started_from_console() -> bool {
    unsafe {
        let console_wnd = GetConsoleWindow();
        let process_id = GetCurrentProcessId();

        let mut parent_id = DWORD::default();
        GetWindowThreadProcessId(console_wnd, &mut parent_id);

        process_id != parent_id
    }
}
