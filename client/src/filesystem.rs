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
    // Base for config, profiles, ...
    static ref BASE_PATH: PathBuf = base();
    // Base for the assets
    static ref ASSETS_PATH: PathBuf = assets();
}

// TODO: Is there a way to figure out whether airshipper has been installed or not
//       to allow to use another base location and avoid polluting the current install while developing?

/// Returns the base path where all airshipper files like config, profiles belong.
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

/// Tries to locate the static assets at various places.
/// Priorities relative over absolute paths (e.g. next to the executable before checking /usr/share/airshipper/.. etc)
fn assets() -> PathBuf {
    let mut paths = Vec::new();

    // Executable path
    if let Ok(mut path) = std::env::current_exe() {
        path.pop();
        paths.push(path);
    }

    // current working directory
    if let Ok(path) = std::env::current_dir() {
        paths.push(path);
    }

    // System paths
    #[cfg(target_os = "linux")]
    paths.push("/usr/share/airshipper/assets".into());

    for path in paths.clone() {
        match find_folder::Search::ParentsThenKids(3, 1)
            .of(path)
            .for_folder("assets")
        {
            Ok(assets_path) => return assets_path,
            Err(_) => continue,
        }
    }

    panic!(
        "Airshipper assets could not be found! Searched folders:\n{})",
        paths.iter().fold(String::new(), |mut a, path| {
            a += &path.to_string_lossy();
            a += "\n";
            a
        }),
    );
}

pub(crate) fn base_path() -> impl std::fmt::Display {
    BASE_PATH.display()
}

pub(crate) fn assets_path() -> impl std::fmt::Display {
    ASSETS_PATH.display()
}

/// Returns path to the file which saves the current state
pub(crate) fn get_savedstate_path() -> PathBuf {
    BASE_PATH.join(SAVED_STATE_FILE)
}

/// Returns path to where the assets are stored
pub(crate) fn get_assets_path(name: &str) -> String {
    ASSETS_PATH.join(name).display().to_string()
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
