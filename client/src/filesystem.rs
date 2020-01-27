//! Deals with all filesystem specific details

use std::path::PathBuf;

#[cfg(windows)]
pub const DOWNLOAD_FILE: &str = "veloren.zip";
#[cfg(unix)]
pub const DOWNLOAD_FILE: &str = "veloren";

#[cfg(windows)]
pub const VOXYGEN_FILE: &str = "veloren-voxygen.exe";
#[cfg(unix)]
pub const VOXYGEN_FILE: &str = "veloren-voxygen";

#[cfg(windows)]
pub const SERVER_CLI_FILE: &str = "veloren-server-cli.exe";
#[cfg(unix)]
pub const SERVER_CLI_FILE: &str = "veloren-server-cli";

const SAVED_STATE_FILE: &str = "airshipper_state.ron";
const LOG_FILE: &str = "airshipper.log";

lazy_static::lazy_static! {
    static ref BASE_PATH: PathBuf = base();
}

// TODO: Is there a way to figure out whether airshipper has been installed or not
//       to allow to use another base location and avoid polluting the current install while developing?

/// Returns the base path where all airshipper files belong
///
/// |Platform | Example                                                       |
/// | ------- | ------------------------------------------------------------- |
/// | Linux   | /home/alice/.local/share/barapp                               |
/// | macOS   | /Users/Alice/Library/Application Support/com.Foo-Corp.Bar-App |
/// | Windows | C:\Users\Alice\AppData\Roaming                                |
fn base() -> PathBuf {
    let path = dirs::data_dir()
        .expect("Couldn't locate where to put launcher data!")
        .join("airshipper");
    std::fs::create_dir_all(&path).expect("failed to create data directory!");
    path
}

pub(crate) fn base_path() -> impl std::fmt::Display {
    BASE_PATH.display()
}

/// Returns path to the file which saves the current state
pub(crate) fn get_savedstate_path() -> PathBuf {
    BASE_PATH.join(SAVED_STATE_FILE)
}

/// Returns path to where the assets are stored
/// TODO: More gracefull assets finding!
pub(crate) fn get_assets_path(name: &str) -> String {
    if BASE_PATH.join("assets").join(name).exists() {
        BASE_PATH.join("assets").join(name).display().to_string()
    } else {
        std::env::current_dir()
            .unwrap()
            .join("client")
            .join("assets")
            .join(name)
            .display()
            .to_string()
    }
}

/// Returns path to a profile while creating the folder
pub(crate) fn get_profile_path(profile_name: &str) -> std::path::PathBuf {
    let path = BASE_PATH.join("profiles").join(profile_name);
    std::fs::create_dir_all(&path).expect("failed to profile directory!");
    path
}

/// Returns path to the file where the logs will be stored
pub(crate) fn get_log_path() -> PathBuf {
    BASE_PATH.join(LOG_FILE)
}
