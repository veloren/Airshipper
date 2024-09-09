use crate::{fs, windows, Result};
use self_update::update::{Release, ReleaseAsset};
use semver::Version;
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    os::windows::ffi::OsStrExt,
    ptr,
};
use windows_sys::Win32::{
    System::{Console::GetConsoleWindow, Threading::GetCurrentProcessId},
    UI::{
        Shell::ShellExecuteW,
        WindowsAndMessaging::{GetWindowThreadProcessId, ShowWindow, SW_HIDE, SW_SHOW},
    },
};

fn get_asset(release: &Release) -> Option<ReleaseAsset> {
    release.asset_for("windows", None).or_else(|| {
        release
            .assets
            .iter()
            .find(|a| {
                let name = a.name.to_uppercase();
                let download_url = a.download_url.to_uppercase();

                download_url.ends_with(".MSI")
                    || download_url.ends_with("INSTALLER.EXE")
                    || (name.ends_with("INSTALLER") && name.contains("WINDOWS"))
            })
            .cloned()
    })
}

pub fn query() -> Result<Option<Release>> {
    let releases = self_update::backends::gitlab::ReleaseList::configure()
        .repo_owner("veloren")
        .repo_name("airshipper")
        .build()?
        .fetch()?;

    // Get latest Github release
    if let Some(latest_release) = releases.first() {
        tracing::trace!("detected online release: {:?}", latest_release);

        let newer = Version::parse(&latest_release.version)?
            > Version::parse(env!("CARGO_PKG_VERSION"))?;
        let contains_asset = get_asset(latest_release).is_some();

        tracing::trace!(?newer, ?contains_asset, "online release info");

        // Check if Github release is newer
        if contains_asset && newer {
            tracing::debug!("Found new Airshipper release: {}", &latest_release.version);
            return Ok(Some(latest_release.clone()));
        } else {
            tracing::debug!("Airshipper is up-to-date.");
        }
    }
    Ok(None)
}

/// Tries to self update with provided release
pub(crate) fn update(latest_release: &Release) -> Result<()> {
    let update_cache_path = fs::get_cache_path().join("update");

    // Cleanup
    let _ = std::fs::remove_dir_all(&update_cache_path);
    std::fs::create_dir_all(&update_cache_path)
        .expect("failed to create cache directory!");

    let asset = get_asset(latest_release);

    // Check Github release provides artifact for current platform
    if let Some(asset) = asset {
        tracing::debug!("Found asset: {:?}", asset);
        let download_file_name = asset
            .download_url
            .rsplit_once('/')
            .map(|(_, end)| end)
            .unwrap_or("installer.exe");

        let install_file_path = update_cache_path.join(download_file_name);
        tracing::debug!(
            "Downloading '{}' to '{}'",
            &asset.download_url,
            install_file_path.display()
        );
        let install_file_path = update_cache_path.join(&download_file_name);

        let install_file = File::create(&install_file_path)?;

        self_update::Download::from_url(&asset.download_url)
            .set_header(
                reqwest::header::ACCEPT,
                "application/octet-stream".parse().unwrap(),
            )
            .show_progress(false)
            .download_to(&install_file)?;

        install_file.sync_all()?; //make sure we block on sync before we start it
        drop(install_file);

        tracing::debug!("Starting installer...");
        // Execute the installer
        let result = match install_file_path.extension().and_then(|f| f.to_str()) {
            Some("exe") => windows::execute_as_admin(install_file_path, ""),
            _ => windows::execute_as_admin(
                "msiexec",
                &format!(
                    "/passive /i \"{}\" /L*V \"{}\" AUTOSTART=1",
                    install_file_path.display(),
                    update_cache_path.join("airshipper-install.log").display()
                ),
            ),
        };

        if result <= 32 {
            tracing::error!(
                "Failed to update airshipper! {}",
                std::io::Error::last_os_error()
            );
        }
        std::process::exit(0);
    }

    Ok(())
}

pub fn execute_as_admin<T, T2>(program: T, args: T2) -> i32
where
    T: Into<OsString>,
    T2: Into<OsString>,
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
        ) as i32
    }
}

/// Hides the console incase the process hasn't been started from one.
pub fn hide_non_inherited_console() {
    if !started_from_console() {
        let window = unsafe { GetConsoleWindow() };
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
        if !window.is_null() {
            unsafe {
                ShowWindow(window, SW_HIDE);
            }
        }
    }
}

/// Determines whether the process has been started from console.
fn started_from_console() -> bool {
    unsafe {
        let console_wnd = GetConsoleWindow();
        let process_id = GetCurrentProcessId();

        let mut parent_id = 0;
        GetWindowThreadProcessId(console_wnd, &mut parent_id);

        process_id != parent_id
    }
}
