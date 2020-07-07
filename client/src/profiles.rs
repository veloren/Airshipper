use crate::{filesystem, network, Result};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, process::Command};

/// Represents a version with channel, name and path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub channel: Channel,

    pub directory: PathBuf,
    pub version: String,
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
    /// Creates a new profile and downloads the correct files into the target directory.
    pub fn new(name: String, channel: Channel) -> Self {
        Self {
            directory: filesystem::get_profile_path(&name),
            name,
            channel,
            version: "".to_owned(), // Will be set by download
        }
    }

    pub fn start_download(&self) -> Result<isahc::Metrics> {
        network::start_download(&self)
    }

    pub async fn install(mut self) -> Result<Profile> {
        let latest_version = self.check_for_update().await?;
        if self.version != latest_version {
            // TODO: maybe let install return the new profile or make it all &mut
            network::install(&self).await?;
            self.version = latest_version;
            Ok(self)
        } else {
            Err("No newer version found".into())
        }
    }

    // TODO: add possibility to start the server too
    pub fn start(&self) -> Result<()> {
        let mut envs = HashMap::new();
        envs.insert("VOXYGEN_CONFIG", self.directory.clone().into_os_string());

        log::debug!("Launching {}", self.voxygen_path().display());
        log::debug!("CWD: {:?}", self.directory);
        log::debug!("ENV: {:?}", envs);

        let cmd = Command::new(self.voxygen_path())
            .current_dir(&self.directory)
            .envs(envs)
            .status()?;
        log::debug!(
            "Veloren exited with code: {}",
            cmd.code()
                .map(|x| x.to_string())
                .unwrap_or_else(|| "Exit code unavailable.".to_string())
        );
        Ok(())
    }

    pub async fn check_for_update(&self) -> Result<String> {
        network::get_version(&self).await
    }

    /// Returns path to voxygen binary.
    /// e.g. <base>/profiles/latest/veloren-voxygen.exe
    fn voxygen_path(&self) -> PathBuf {
        self.directory.join(filesystem::VOXYGEN_FILE)
    }
}
