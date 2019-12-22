use crate::config::ClientConfig;
use crate::models::{Channel, Profile};
use crate::Result;
use reqwest::header::*;
use reqwest::{Client, ClientBuilder};
use std::path::PathBuf;

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
pub fn version(config: &ClientConfig, profile: &Profile) -> Result<String> {
    let mut resp = CLIENT
        .get(&config.get_version_uri(profile.channel))
        .send()?;
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

/// Downloads and unzips to destination
pub fn download(config: &ClientConfig, destination: &PathBuf, channel: &Channel) -> Result<()> {
    use crate::interface::downloadbar::{download_with_progress, unzip_with_progress};

    // Download
    std::fs::create_dir_all(destination)?;
    let zip_file = download_with_progress(
        &CLIENT,
        &config.get_artifact_uri(channel),
        &destination.join(crate::config::DOWNLOAD_FILE),
    )?;

    // Extract
    log::debug!("Unzipping artifacts to {}", destination.display());
    unzip_with_progress(zip_file, &config.base_path, &destination)?;

    // Delete downloaded zip
    log::trace!("Extracted files, deleting zip archive.");
    std::fs::remove_file(destination.join(crate::config::DOWNLOAD_FILE))?;

    #[cfg(unix)]
    set_permissions(vec![
        &destination.join(crate::config::VOXYGEN_FILE),
        &destination.join(crate::config::SERVER_CLI_FILE),
    ])?;

    Ok(())
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
