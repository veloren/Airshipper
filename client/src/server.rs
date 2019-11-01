use crate::config::ClientConfig;
use crate::models::{Channel, Profile};
use crate::Result;
use reqwest::header::*;
use reqwest::{Client, ClientBuilder};
use std::fs::OpenOptions;
use std::path::PathBuf;

lazy_static::lazy_static! {
    static ref CLIENT: Client = {
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

/// TODO: Add a progress bar
/// Downloads and unzips to destination
pub fn download(config: &ClientConfig, destination: &PathBuf, channel: &Channel) -> Result<()> {
    let mut resp = CLIENT.get(&config.get_artifact_uri(channel)).send()?;
    std::fs::create_dir_all(destination)?;

    if resp.status().is_success() {
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .append(false)
            .truncate(true)
            .open(destination.join(crate::config::DOWNLOAD_FILE))?;
        std::io::copy(&mut resp, &mut f)?;
        log::debug!("Unzipping artifacts to {}", destination.display());
        let mut archive = zip::ZipArchive::new(&mut f)?;

        for i in 1..archive.len() {
            let mut file = archive.by_index(i)?;
            let path = destination.join(file.sanitized_name());

            if file.is_dir() {
                std::fs::create_dir_all(path)?;
            } else {
                log::trace!("Unzipping {}", path.display());
                let mut target = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)?;

                std::io::copy(&mut file, &mut target)?;
            }
        }

        // Delete downloaded zip
        log::trace!("Extracted files, deleting zip archive.");
        std::fs::remove_file(destination.join(crate::config::DOWNLOAD_FILE))?;

        #[cfg(unix)]
        set_permissions(vec![
            &destination.join(crate::config::VOXYGEN_FILE),
            &destination.join(crate::config::SERVER_CLI_FILE),
        ])?;

        Ok(())
    } else {
        Err(format!(
            "Failed to download version. Server returned '{}'",
            resp.status()
        )
        .into())
    }
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
