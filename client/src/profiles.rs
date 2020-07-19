use crate::{consts, fs, net, CommandBuilder, Result};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// Represents a version with channel, name and path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub channel: Channel,

    pub directory: PathBuf,
    pub version: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Profile::new("default".to_owned(), Channel::Nightly)
    }
}

#[derive(Debug, Display, Clone, Copy, Serialize, Deserialize)]
pub enum Channel {
    Nightly,
    /* TODO: Release,
     * TODO: Source, */
}

impl Profile {
    pub fn new(name: String, channel: Channel) -> Self {
        Self {
            directory: fs::get_profile_path(&name),
            name,
            channel,
            version: None,
        }
    }
    /// Returns path to voxygen binary.
    /// e.g. <base>/profiles/default/veloren-voxygen.exe
    fn voxygen_path(&self) -> PathBuf {
        self.directory.join(consts::VOXYGEN_FILE)
    }

    /// Returns the download url for this profile
    pub fn url(&self) -> String {
        format!(
            "{}/latest/{}/{}",
            consts::DOWNLOAD_SERVER,
            std::env::consts::OS,
            self.channel
        )
    }

    pub fn download_path(&self) -> PathBuf {
        self.directory.join(consts::DOWNLOAD_FILE)
    }

    fn version_url(&self) -> String {
        format!(
            "{}/version/{}/{}",
            consts::DOWNLOAD_SERVER,
            std::env::consts::OS,
            self.channel
        )
    }

    // TODO: add possibility to start the server too
    pub fn start(profile: Profile) -> CommandBuilder {
        let mut envs = HashMap::new();
        envs.insert("VOXYGEN_CONFIG", profile.directory.clone().into_os_string());
        // TODO: make use of all Veloren env vars!

        log::debug!("Launching {}", profile.voxygen_path().display());
        log::debug!("CWD: {:?}", profile.directory);
        log::debug!("ENV: {:?}", envs);

        let mut cmd = CommandBuilder::new(profile.voxygen_path());
        cmd.current_dir(&profile.directory);
        cmd.envs(envs);

        cmd
    }

    pub async fn update(profile: Profile) -> Result<Option<String>> {
        let remote = net::query(&profile.version_url()).await?.text().await?;

        if remote != profile.version.unwrap_or_default() {
            Ok(Some(remote))
        } else {
            Ok(None)
        }
    }

    pub async fn install(mut profile: Profile, version: String) -> Result<Profile> {
        tokio::task::block_in_place(|| fs::unzip(&profile))?;

        #[cfg(unix)]
        set_permissions(vec![
            &profile.directory.join(consts::VOXYGEN_FILE),
            &profile.directory.join(consts::SERVER_CLI_FILE),
        ])
        .await?;
        // After successful install, update the profile.
        profile.version = Some(version);

        Ok(profile)
    }
}

/// Tries to set executable permissions on linux
#[cfg(unix)]
async fn set_permissions(files: Vec<&std::path::PathBuf>) -> Result<()> {
    use tokio::process::Command;

    for file in files {
        Command::new("chmod").arg("+x").arg(file).spawn()?.await?;
    }
    Ok(())
}
