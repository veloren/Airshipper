use crate::{fs, windows, Result};
use semver::Version;
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    os::windows::ffi::OsStrExt,
    ptr,
};
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

/// Tries to self update incase a newer version got released.
pub(crate) fn update() -> Result<()> {
    // Cleanup
    let _ = std::fs::remove_dir_all(fs::get_cache_path());

    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("songtronix")
        .repo_name("airshipper")
        .build()?
        .fetch()?;

    // Get latest Gtihub release
    if let Some(latest_release) = releases.first() {
        // Check if Github release is newer
        if Version::parse(&latest_release.version)?
            > Version::parse(env!("CARGO_PKG_VERSION"))?
        {
            log::debug!("Found new Airshipper release: {}", latest_release.version);
            // Check Github release provides artifact for current platform
            let asset = latest_release
                .asset_for("windows")
                .or_else(|| latest_release.asset_for(".msi"));

            if let Some(asset) = asset {
                log::debug!("Found asset: {:?}", asset);
                log::info!(
                    "Found airshipper update! It's highly recommended to update. \
                     Install? [Y/n]"
                );
                if crate::cli::confirm_action()? {
                    log::debug!(
                        "Downloading '{}' to '{}'",
                        &asset.download_url,
                        fs::get_cache_path().join(&asset.name).display()
                    );
                    let msi_file_path = fs::get_cache_path().join(&asset.name);
                    std::fs::create_dir_all(fs::get_cache_path())?;

                    let msi_file = File::create(&msi_file_path)?;

                    self_update::Download::from_url(&asset.download_url)
                        .show_progress(true)
                        .download_to(&msi_file)?;

                    // Extract installer incase it's zipped
                    if asset.name.ends_with(".zip") {
                        log::debug!("Extracting asset...");
                        self_update::Extract::from_source(&msi_file_path)
                            .archive(self_update::ArchiveKind::Zip)
                            .extract_file(
                                &fs::get_cache_path(),
                                asset.name.strip_suffix(".zip").unwrap(),
                            )?;
                    }

                    drop(msi_file);

                    log::debug!("Starting installer...");
                    // Execute msi installer
                    let result = windows::execute_as_admin(
                        "msiexec",
                        &format!(
                            "/passive /i {} /L*V {} AUTOSTART=1",
                            msi_file_path.display(),
                            fs::get_cache_path()
                                .join("airshipper-install.log")
                                .display()
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
            } else {
                log::debug!("Release does not contain required target asset.");
            }
        } else {
            log::debug!("Airshipper is up-to-date.");
        }
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
