use crate::{consts, fs, net, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ffi::OsString, path::PathBuf};
use tokio::process::Command;

// TODO: Support multiple profiles and manage them here.

/// Represents a version with channel, name and path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub channel: Channel,
    // FIXME: This field is currently ignored.
    // Persisting the storage path led to problems with the snap package because
    // the directory the snap is allowed to write to changes with each new snap version.
    // Since there is currently no use in persisting this path anyway, we ignore it.
    // It is not removed either to guarantee backwards-compatibility by making sure
    // configuration files containing this field can still be successfully parsed
    #[serde(rename = "directory")]
    _directory: PathBuf,
    pub version: Option<String>,
}

impl Default for Profile {
    fn default() -> Self {
        Profile::new("default".to_owned(), Channel::Nightly)
    }
}

#[derive(Debug, derive_more::Display, Clone, Copy, Serialize, Deserialize)]
pub enum Channel {
    Nightly,
    /* TODO: Release,
     * TODO: Source, */
}

impl Profile {
    pub fn new(name: String, channel: Channel) -> Self {
        Self {
            _directory: fs::profile_path(&name),
            name,
            channel,
            version: None,
        }
    }

    pub fn directory(&self) -> PathBuf {
        fs::profile_path(&self.name)
    }

    /// Returns path to voxygen binary.
    /// e.g. <base>/profiles/default/veloren-voxygen.exe
    fn voxygen_path(&self) -> PathBuf {
        self.directory().join(consts::VOXYGEN_FILE)
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
        self.directory().join(consts::DOWNLOAD_FILE)
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
    pub fn start(profile: &Profile, verbosity: i32) -> Command {
        let mut envs = HashMap::new();
        let userdata_dir = profile.directory().join("userdata").into_os_string();
        let screenshot_dir = profile.directory().join("screenshots").into_os_string();
        let assets_dir = profile.directory().join("assets").into_os_string();

        let verbosity = match verbosity {
            0 => OsString::from("info"),
            1 => OsString::from("debug"),
            _ => OsString::from("trace"),
        };

        envs.insert("VOXYGEN_SCREENSHOT", &screenshot_dir);
        envs.insert("VELOREN_USERDATA", &userdata_dir);
        envs.insert("VELOREN_ASSETS", &assets_dir);
        envs.insert("RUST_LOG", &verbosity);

        log::debug!("Launching {}", profile.voxygen_path().display());
        log::debug!("CWD: {:?}", profile.directory());
        log::debug!("ENV: {:?}", envs);

        let mut cmd = Command::new(profile.voxygen_path());
        cmd.current_dir(&profile.directory());
        cmd.envs(envs);

        cmd
    }

    pub async fn update(profile: Profile) -> Result<Option<String>> {
        let remote = net::query(&profile.version_url()).await?.text().await?;

        if remote != profile.version.clone().unwrap_or_default() || !profile.installed() {
            Ok(Some(remote))
        } else {
            Ok(None)
        }
    }

    pub async fn install(mut profile: Profile, version: String) -> Result<Profile> {
        tokio::task::block_in_place(|| fs::unzip(&profile))?;

        #[cfg(unix)]
        set_permissions(vec![
            &profile.directory().join(consts::VOXYGEN_FILE),
            &profile.directory().join(consts::SERVER_CLI_FILE),
        ])
        .await?;
        // After successful install, update the profile.
        profile.version = Some(version);

        Ok(profile)
    }

    /// Returns whether the profile is ready to be started
    pub fn installed(&self) -> bool {
        self.voxygen_path().exists() && self.version.is_some()
    }
}

/// Tries to set executable permissions on linux
#[cfg(unix)]
async fn set_permissions(files: Vec<&std::path::PathBuf>) -> Result<()> {
    for file in files {
        Command::new("chmod").arg("+x").arg(file).spawn()?.await?;
    }
    Ok(())
}
