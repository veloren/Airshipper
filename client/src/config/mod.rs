use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::models::{Channel, Profile};
use crate::Result;

#[cfg(windows)]
pub const VOXYGEN_FILE: &str = "veloren-voxygen.exe";
#[cfg(unix)]
pub const VOXYGEN_FILE: &str = "veloren-voxygen";

//#[cfg(windows)]
//pub const SERVER_CLI_FILE: &str = "veloren-server-cli.exe";
#[cfg(unix)]
pub const SERVER_CLI_FILE: &str = "veloren-server-cli";

#[cfg(windows)]
pub const DOWNLOAD_FILE: &str = "veloren.zip";
#[cfg(unix)]
pub const DOWNLOAD_FILE: &str = "veloren";

pub const PROFILES: &str = "profiles";
pub const LAUNCHER_LOG: &str = "launcher.log";
pub const CONFIG_FILE: &str = "airshipper.ron";

/// Configuration and defaults for airshipper.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ClientConfig {
    pub base_path: PathBuf,
    pub log_file: PathBuf,

    base_url: String,

    pub profiles: Vec<Profile>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        // Get the current platform we run on.
        let base_path = base();
        let log_file = base_path.join(LAUNCHER_LOG);

        let base_url = "https://download.veloren.net".into();

        Self {
            base_path,
            log_file,
            base_url,
            profiles: vec![Profile::default()],
        }
    }
}

impl ClientConfig {
    pub fn load() -> Self {
        // Try loading the configuration from disk
        if let Ok(file) = std::fs::File::open(base().join(CONFIG_FILE)) {
            match ron::de::from_reader(file) {
                Ok(c) => return c,
                Err(e) => {
                    // Failed parsing
                    println!("Failed parsing config: {}", e);
                    println!("Exiting...");
                    std::process::exit(-1);
                }
            }
        }
        // File does not exist.
        let default = Self::default();
        default.save_to_file();
        default
    }

    pub fn save_to_file(&self) {
        use ron::ser::{to_string_pretty, PrettyConfig};
        use std::fs::File;
        use std::io::Write;

        let mut config_file =
            File::create(&self.base_path.join(CONFIG_FILE)).expect("Failed to create config file!");
        let serialised: &str =
            &to_string_pretty(self, PrettyConfig::default()).expect("Failed to serialise config!");
        config_file
            .write_all(serialised.as_bytes())
            .expect("failed writing config!");
    }

    pub fn update(&mut self) -> Result<()> {
        // Avoiding the borrow checker obviously
        self.profiles[0] = self.profiles[0].update(&self)?;
        self.save_to_file();
        Ok(())
    }

    pub fn get_version_uri(&self, channel: Channel) -> String {
        format!(
            "{}/version/{}/{}",
            self.base_url,
            whoami::platform(),
            channel
        )
    }

    pub fn get_artifact_uri(&self, channel: &Channel) -> String {
        format!(
            "{}/latest/{}/{}",
            self.base_url,
            whoami::platform(),
            channel
        )
    }

    pub fn start(&self) -> Result<()> {
        Ok(self.profiles[0].start()?)
    }
}

/// Detects if airshipper has been installed by checking for the binary name.
/// airshipper => use cwd (probably cloned repo/downloaded)
/// veloren => use OS specific location
pub fn base() -> std::path::PathBuf {
    match std::env::current_exe()
        .expect("Couldn't find location of binary!")
        .file_name()
    {
        Some(name) => {
            // Airshipper has been installed
            if name.to_string_lossy().to_lowercase().contains("veloren") {
                let base = dirs::config_dir()
                    .expect("Couldn't locate where to put configuration!")
                    .join("airshipper");

                std::fs::create_dir_all(&base).expect("failed to create data directory!");

                base
            } else {
                // Airshipper hasn't been installed. Use cwd.
                let path =
                    std::env::current_dir().expect("Failed to get current working directory!");
                path
            }
        }
        None => {
            let path = std::env::current_dir().expect("Failed to get current working directory!");
            path
        }
    }
}
