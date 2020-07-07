use crate::{filesystem, network, Result};
use std::ffi::OsStr;

// TODO: We should remove the installer after successful update!

pub(crate) async fn update() -> Result<()> {
    // Note: this will ignore network errors silently.
    if let Some(url) = network::check_win_update().await.ok().flatten() {
        log::info!(
            "Found airshipper update! It's highly recommended to update. Install? [Y/n]"
        );
        if crate::cli::confirm_action()? {
            let mut resp = network::request(&url).await?;
            let path = filesystem::get_cache_path();

            let filename = match url::Url::parse(&url)?
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
                use std::{os::windows::ffi::OsStrExt, ptr};
                use winapi::{
                    ctypes::c_int,
                    um::{shellapi::ShellExecuteW, winuser::SW_SHOW},
                };

                log::debug!(
                    "Download airshipper update to: {}",
                    path.join(&filename).display()
                );
                let mut file = std::fs::File::create(&path.join(&filename))?;
                std::io::copy(&mut resp.body_mut(), &mut file)?;

                // Free up access to file.
                drop(file);

                // Execute msi installer
                let operation: Vec<u16> = OsStr::new("runas\0").encode_wide().collect();
                let bin = OsStr::new("msiexec\0").encode_wide().collect::<Vec<u16>>();

                let arguments: Vec<u16> = OsStr::new(&format!(
                    "/passive /i {} /L*V {} AUTOSTART=1\0",
                    path.join(&filename).display(),
                    path.join("airshipper-install.log").display()
                ))
                .encode_wide()
                .collect();

                let result = unsafe {
                    ShellExecuteW(
                        ptr::null_mut(),
                        operation.as_ptr(),
                        bin.as_ptr(),
                        arguments.as_ptr(),
                        ptr::null(),
                        SW_SHOW,
                    )
                };
                if !(result as c_int > 32) {
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
