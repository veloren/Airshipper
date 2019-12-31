use crate::network;
use crate::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Represents a version with channel, name and path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub channel: Channel,
    pub base_server_url: String,

    pub directory: PathBuf,
    pub version: String,
}

#[derive(Debug, Display, Clone, Copy, Serialize, Deserialize)]
pub enum Channel {
    Nightly,
    // TODO: Release,
    // TODO: Source,
}

impl Profile {
    /// Creates a new profile and downloads the correct files into the target directory.
    pub fn from_download(
        name: String,
        channel: Channel,
        base_server_url: String,
        target: PathBuf,
    ) -> Result<Self> {
        // TODO: check if dir is empty but available
        let mut directory = target;
        directory.push(name.clone());
        let profile = Self {
            name,
            channel,
            base_server_url,
            directory,
            version: "".to_owned(), // Will be set by download
        };

        log::info!("Downloading {} - {}", profile.name, profile.channel);
        network::download(&profile)?;
        Ok(profile)
    }

    /// Returns an updated version of itself if applicable
    ///
    /// NOTE: Don't forget to save the changes!
    /// TODO: Solve this better
    pub fn update(&mut self) -> Result<()> {
        let remote_version = network::get_newest_version_name(&self)?;
        let mut updated = self.clone();

        log::debug!("remote version of {} is {}", self.channel, &remote_version);
        if self.version != remote_version {
            if !self.voxygen_path().exists() {
                log::warn!("Previous installation not found!");
                log::info!("Downloading {} - {}", self.name, self.channel);
                network::download(&self)?;
                self.version = remote_version;
            } else {
                log::info!("Updating...");
                network::download(&self)?;
                self.version = remote_version;
            }
        } else {
            log::info!("Veloren is up-to-date.");
        }
        Ok(())
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
                .unwrap_or("Exit code unavailable.".to_string())
        );
        Ok(())
    }

    /// Returns path to voxygen binary.
    /// e.g. <base>/profiles/latest/veloren-voxygen.exe
    fn voxygen_path(&self) -> PathBuf {
        self.directory.join(crate::VOXYGEN_FILE)
    }
}
