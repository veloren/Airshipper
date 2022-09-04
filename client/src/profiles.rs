#[cfg(unix)]
use crate::nix;
use crate::{channels::Channel, consts, fs, net, server_list, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ffi::OsString, path::PathBuf};
use tokio::process::Command;

// TODO: Support multiple profiles and manage them here.

/// Represents a version with channel, name and path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub server: Server,
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
    pub wgpu_backend: WgpuBackend,
    pub log_level: LogLevel,
    pub env_vars: String,
}

const DEFAULT_PROFILE_NAME: &str = "default";
impl Default for Profile {
    fn default() -> Self {
        Profile::new(
            DEFAULT_PROFILE_NAME.to_owned(),
            Server::Production,
            Channel("weekly".to_owned()),
        )
    }
}

#[derive(
    Debug, derive_more::Display, Clone, Copy, Serialize, Deserialize, PartialEq, Eq,
)]
pub enum WgpuBackend {
    Auto,
    DX11,
    DX12,
    Metal,
    Vulkan,
}

#[cfg(target_os = "windows")]
pub static WGPU_BACKENDS: &[WgpuBackend] = &[
    WgpuBackend::Auto,
    WgpuBackend::DX11,
    WgpuBackend::DX12,
    WgpuBackend::Vulkan,
];

#[cfg(target_os = "linux")]
pub static WGPU_BACKENDS: &[WgpuBackend] = &[WgpuBackend::Auto, WgpuBackend::Vulkan];

#[cfg(target_os = "macos")]
pub static WGPU_BACKENDS: &[WgpuBackend] = &[WgpuBackend::Auto, WgpuBackend::Metal];

#[derive(
    Debug, derive_more::Display, Clone, Copy, Serialize, Deserialize, PartialEq, Eq,
)]
pub enum Server {
    Production,
    Staging,
    Test,
}

pub static SERVERS: &[Server] = &[Server::Production, Server::Staging, Server::Test];

#[derive(
    Debug, derive_more::Display, Clone, Copy, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum LogLevel {
    Default,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Default
    }
}

pub static LOG_LEVELS: &[LogLevel] =
    &[LogLevel::Default, LogLevel::Debug, LogLevel::Trace];

impl Server {
    pub fn url(&self) -> &str {
        match self {
            Server::Production => "https://download.veloren.net",
            Server::Staging => "https://download.staging.veloren.net",
            Server::Test => "https://download.test.veloren.net",
        }
    }
}

impl Profile {
    pub fn new(name: String, server: Server, channel: Channel) -> Self {
        Self {
            _directory: fs::profile_path(&name),
            name,
            server,
            channel,
            version: None,
            wgpu_backend: WgpuBackend::Auto,
            log_level: LogLevel::Default,
            env_vars: String::new(),
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

    /// Returns path to the voxygen logs directory
    /// e.g. <base>/profiles/default/logs
    pub fn voxygen_logs_path(&self) -> PathBuf {
        self.directory().join(consts::LOGS_DIR)
    }

    /// Returns the download url for this profile
    pub fn url(&self) -> String {
        format!(
            "{}/latest/{}/{}/{}",
            self.server.url(),
            std::env::consts::OS,
            std::env::consts::ARCH,
            self.channel
        )
    }

    pub fn download_path(&self) -> PathBuf {
        self.directory().join(consts::DOWNLOAD_FILE)
    }

    fn version_url(&self) -> String {
        format!(
            "{}/version/{}/{}/{}",
            self.server.url(),
            std::env::consts::OS,
            std::env::consts::ARCH,
            self.channel
        )
    }

    pub(crate) fn channel_url(&self) -> String {
        format!(
            "{}/channels/{}/{}",
            self.server.url(),
            std::env::consts::OS,
            std::env::consts::ARCH,
        )
    }

    pub(crate) fn api_version_url(&self) -> String {
        format!("{}/api/version", self.server.url(),)
    }

    pub(crate) fn announcement_url(&self) -> String {
        format!("{}/announcement", self.server.url(),)
    }

    // TODO: add possibility to start the server too
    pub fn start(profile: &Profile, game_server_address: Option<&str>) -> Command {
        let mut envs = HashMap::new();
        let userdata_dir = profile.directory().join("userdata").into_os_string();
        let screenshot_dir = profile.directory().join("screenshots").into_os_string();
        let assets_dir = profile.directory().join("assets").into_os_string();

        if profile.log_level != LogLevel::Default {
            let log_level = match profile.log_level {
                LogLevel::Default => OsString::from("info"),
                LogLevel::Debug => OsString::from("debug"),
                LogLevel::Trace => OsString::from("trace"),
            };
            envs.insert("RUST_LOG", log_level);
        }

        envs.insert("VOXYGEN_SCREENSHOT", screenshot_dir);
        envs.insert("VELOREN_USERDATA", userdata_dir);
        envs.insert("VELOREN_ASSETS", assets_dir);

        if profile.wgpu_backend != WgpuBackend::Auto {
            let wgpu_backend = match profile.wgpu_backend {
                WgpuBackend::DX11 => "dx11",
                WgpuBackend::DX12 => "dx12",
                WgpuBackend::Metal => "metal",
                WgpuBackend::Vulkan => "vulkan",
                _ => unreachable!(
                    "Unsupported WgpuBackend value: {}",
                    profile.wgpu_backend
                ),
            };
            envs.insert("WGPU_BACKEND", OsString::from(wgpu_backend));
        }

        let (env_vars, env_var_errors) = parse_env_vars(&profile.env_vars);
        for err in env_var_errors {
            tracing::error!("Environment variable error: {}", err);
        }
        for (var, value) in env_vars {
            envs.insert(var, OsString::from(value));
        }

        tracing::debug!("Launching {}", profile.voxygen_path().display());
        tracing::debug!("CWD: {:?}", profile.directory());
        tracing::debug!("ENV: {:?}", envs);

        let mut cmd = Command::new(profile.voxygen_path());
        cmd.current_dir(&profile.directory());
        cmd.envs(envs);

        // If a server is selected in the server browser pass it through to Voxygen
        if let Some(game_server_address) = game_server_address {
            cmd.args(&["--server", game_server_address]);
        }

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
        {
            let voxygen_file = profile.directory().join(consts::VOXYGEN_FILE);
            let server_cli_file = profile.directory().join(consts::SERVER_CLI_FILE);

            // Patch executable files if we are on NixOS
            if nix::is_nixos()? {
                tokio::task::block_in_place(|| {
                    nix::patch_elf(&voxygen_file, &server_cli_file)
                })?;
            } else {
                set_permissions(vec![&voxygen_file, &server_cli_file]).await?;
            }
        }

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
        Command::new("chmod")
            .arg("+x")
            .arg(file)
            .spawn()?
            .wait()
            .await?;
    }
    Ok(())
}

fn parse_env_vars(env_vars: &str) -> (Vec<(&str, &str)>, Vec<String>) {
    let mut errors = Vec::new();
    let vars = env_vars
        .split(',')
        .filter_map(|var| {
            if let Some((key, value)) = var.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                if key.chars().count() == 0 {
                    errors.push(format!("Invalid variable '{}'", key))
                }
                Some((key, value))
            } else {
                if var.chars().count() == 0 {
                    errors.push("Unnecessary ',' in variable list".to_string());
                } else {
                    errors.push(format!("Variable '{}' has no corresponding value", var));
                }
                None
            }
        })
        .collect();

    (vars, errors)
}
