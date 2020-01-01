use crate::profiles::Profile;
use crate::Result;
use reqwest::header::*;
use reqwest::{Client, ClientBuilder};
use std::path::PathBuf;

#[cfg(windows)]
pub const DOWNLOAD_FILE: &str = "veloren.zip";
#[cfg(unix)]
pub const DOWNLOAD_FILE: &str = "veloren";

// Maybe move this over to downloadbar ?
lazy_static::lazy_static! {
    pub static ref CLIENT: Client = {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            format!("Airshipper/{} ({})", env!("CARGO_PKG_VERSION"), whoami::os())
                .parse()
                .unwrap(),
        );
        ClientBuilder::new()
            .connect_timeout(std::time::Duration::from_secs(30))
            .default_headers(headers)
            .build().expect("FATAL: failed to build reqwest client!")
    };
}

/// Returns the remote version of the profile
pub fn get_newest_version_name(profile: &Profile) -> Result<String> {
    let mut resp = CLIENT.get(&get_version_uri(profile)).send()?;
    if resp.status().is_success() {
        Ok(resp.text()?)
    } else {
        Err(format!(
            "Couldn't download version information. Server returned '{}'",
            resp.status()
        )
        .into())
    }
}

/// Downloads and unzips to target directory
pub fn download(profile: &Profile) -> Result<()> {
    use crate::downloadbar::{download_with_progress, unzip_with_progress};

    // Download
    std::fs::create_dir_all(&profile.directory)?;
    let zip_file = download_with_progress(
        &CLIENT,
        &get_artifact_uri(&profile),
        &profile.directory.join(DOWNLOAD_FILE),
    )?;

    // Extract
    log::debug!("Unzipping artifacts to {:?}", profile.directory);
    unzip_with_progress(zip_file, &profile.directory)?;

    // Delete downloaded zip
    log::trace!("Extracted files, deleting zip archive.");
    std::fs::remove_file(profile.directory.join(DOWNLOAD_FILE))?;

    #[cfg(unix)]
    set_permissions(vec![
        &profile.directory.join(crate::VOXYGEN_FILE),
        &profile.directory.join(crate::SERVER_CLI_FILE),
    ])?;

    Ok(())
}

fn get_version_uri(profile: &Profile) -> String {
    format!(
        "{}/version/{}/{}",
        profile.base_server_url,
        whoami::platform(),
        profile.channel
    )
}
fn get_artifact_uri(profile: &Profile) -> String {
    format!(
        "{}/latest/{}/{}",
        profile.base_server_url,
        whoami::platform(),
        profile.channel
    )
}

/// Tries to set executable permissions on linux
#[cfg(unix)]
fn set_permissions(files: Vec<&PathBuf>) -> Result<()> {
    use std::process::Command;
    for file in files {
        Command::new("chmod").arg("+x").arg(file).spawn()?;
    }
    Ok(())
}
