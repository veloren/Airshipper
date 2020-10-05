//! Deals with all filesystem specific details

use crate::{consts, profiles::Profile, Result};
use std::path::PathBuf;

lazy_static::lazy_static! {
    // Base for config, profiles, ...
    static ref BASE_PATH: PathBuf = base();
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

#[cfg(windows)]
pub fn get_cache_path() -> PathBuf {
    dirs::cache_dir().unwrap().join(env!("CARGO_PKG_NAME"))
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

/// Extracts downloaded zip file and deletes it after successful extraction.
///
/// Note: it's synchronous!
pub fn unzip(profile: &Profile) -> Result<()> {
    log::info!("Unzipping to {:?}", profile.directory);
    let mut zip_file =
        std::fs::File::open(&profile.directory.join(consts::DOWNLOAD_FILE))?;

    let mut archive = zip::ZipArchive::new(&mut zip_file)?;

    // Delete all assets to ensure that no obsolete assets will remain.
    if profile.directory.join("assets").exists() {
        std::fs::remove_dir_all(profile.directory.join("assets"))?;
    }

    for i in 1..archive.len() {
        let mut file = archive.by_index(i)?;
        // TODO: Verify that `sanitized_name()` works correctly in this case.
        #[allow(deprecated)]
        let path = profile.directory.join(file.sanitized_name());

        if file.is_dir() {
            std::fs::create_dir_all(path)?;
        } else {
            let mut target = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;

            std::io::copy(&mut file, &mut target)?;
        }
    }

    // Delete downloaded zip
    log::trace!("Extracted files, deleting zip archive.");
    std::fs::remove_file(profile.directory.join(consts::DOWNLOAD_FILE))?;

    Ok(())
}
