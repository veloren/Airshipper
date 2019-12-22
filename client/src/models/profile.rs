use crate::config::ClientConfig;
use crate::server;
use crate::Result;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Represents a version with channel, name and path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    name: String,
    cwd: PathBuf,

    pub channel: Channel,
    version: String,
}

#[derive(Debug, Display, Clone, Copy, Serialize, Deserialize)]
pub enum Channel {
    Nightly,
    // TODO: Release,
    // TODO: Source,
}

impl Default for Profile {
    fn default() -> Self {
        let name = "latest".to_string();
        let cwd = crate::config::base()
            .join(crate::config::PROFILES)
            .join(&name);
        let channel = Channel::Nightly;
        let version = String::new();

        Self {
            name,
            cwd,
            channel,
            version,
        }
    }
}

/// Will read from stdin for confirmation
/// NOTE: no input = true
/// Temporary...
fn confirm_action(name: &str) -> Result<bool> {
    log::info!("Update for '{}' profile found. Download? [Y/n] ", name);
    let mut buffer = String::new();
    let _ = std::io::stdin().read_line(&mut buffer)?;
    buffer = buffer.to_lowercase();

    if buffer.trim().is_empty() {
        Ok(true)
    } else if buffer.starts_with("y") {
        Ok(true)
    } else if buffer.starts_with("n") {
        Ok(false)
    } else {
        // for the accidental key smash
        Ok(false)
    }
}

impl Profile {
    /// Returns an updated version of itself if applicable
    ///
    /// NOTE: Don't forget to save the changes!
    /// TODO: Solve this better
    pub fn update(&self, config: &ClientConfig) -> Result<Self> {
        let remote_version = server::version(&config, &self)?;
        let mut updated = self.clone();

        log::debug!("remote version of {} is {}", self.channel, &remote_version);
        if self.version != remote_version {
            if !self.voxygen_path().exists() {
                log::info!("Downloading {} - {}", self.name, self.channel);
                server::download(&config, &self.cwd, &self.channel)?;
                updated.version = remote_version;
                return Ok(updated);
            } else if confirm_action(&self.name)? {
                log::info!("Updating...");
                server::download(&config, &self.cwd, &self.channel)?;
                updated.version = remote_version;
                return Ok(updated);
            }
        } else {
            log::info!("Veloren is up-to-date.");
        }
        Ok(updated)
    }

    // TODO: add possibility to start the server too
    pub fn start(&self) -> Result<()> {
        let mut envs = HashMap::new();
        envs.insert("VOXYGEN_CONFIG", self.cwd.clone().into_os_string());

        log::debug!("Launching {}", self.voxygen_path().display());
        log::debug!("CWD: {}", self.cwd.display());
        log::debug!("ENV: {:?}", envs);

        let cmd = Command::new(self.voxygen_path())
            .current_dir(&self.cwd)
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
        self.cwd.join(crate::config::VOXYGEN_FILE)
    }
}
