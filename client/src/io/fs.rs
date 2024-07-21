//! Deals with all filesystem specific details

use crate::consts;
use std::path::{Path, PathBuf};

lazy_static::lazy_static! {
    // Base for config, profiles, ...
    pub static ref BASE_PATH: PathBuf = base();
}

// TODO: Is there a way to figure out whether airshipper has been installed or not
//       to allow to use another base location and avoid polluting the current install
// while developing?

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

pub fn base_path() -> impl std::fmt::Display {
    BASE_PATH.display()
}

pub fn get_cache_path() -> PathBuf {
    let cache_path = dirs::cache_dir()
        .expect("Couldn't find OS cache directory")
        .join(env!("CARGO_PKG_NAME"));
    std::fs::create_dir_all(&cache_path).expect("failed to create cache directory!");
    cache_path
}

/// Returns path to the file which saves the current state
pub fn savedstate_file() -> PathBuf {
    BASE_PATH.join(consts::SAVED_STATE_FILE)
}

/// Returns path to a profile while creating the folder
pub fn profile_path(profile_name: &str) -> PathBuf {
    let path = BASE_PATH.join("profiles").join(profile_name);
    std::fs::create_dir_all(&path).expect("failed to profile directory!"); // TODO
    path
}

/// Returns path to the file where the logs will be stored
pub fn log_file() -> PathBuf {
    BASE_PATH.join(consts::LOG_FILE)
}

/// Returns log-directory and log-file
pub fn log_path_file() -> (&'static Path, &'static str) {
    (&BASE_PATH, consts::LOG_FILE)
}
