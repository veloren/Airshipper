use derive_more::Display;
use rocket::{config::*, Rocket};
use std::convert::TryFrom;

/// The project ID of veloren on gitlab.
pub const PROJECT_ID: u64 = 10174980;
/// The Hook Type which gets parsed for artifacts.
pub const HOOK_TYPE: &str = "Pipeline Hook";

/// File ending of *downloaded* artifacts
pub const WINDOWS_FILE_ENDING: &str = "zip";
pub const LINUX_FILE_ENDING: &str = "zip";

pub const DATABASE_FILE: &str = "airshipper.db";

/// Configuration and defaults for the entire server.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub address: std::net::SocketAddr,
    /// Off, Normal, Debug, Critical
    pub log_level: LogLevel,
    /// Development, Staging, Production
    pub rocket_env: Env,
    /// DO Space details
    pub bucket_name: String,
    pub bucket_endpoint: String,
    pub bucket_region: s3::region::Region,
    pub bucket_access_key: String,
    pub bucket_secret_key: String,

    /// Specified secret to verify webhook is from gitlab
    pub gitlab_secret: String,
    /// At which stage of the pipeline the artifacts are uploaded.
    pub artifact_stage: String,
    /// What branch should be downloaded
    pub target_branch: String,
    /// What binary build[s] should be downloaded
    /// NOTE: These names have to include the OS!
    pub target_executable: Vec<String>,
}

impl ServerConfig {
    pub fn load() -> crate::Result<Self> {
        Ok(Self {
            gitlab_secret: Self::expect_env_key("AIRSHIPPER_GITLAB_SECRET"),
            artifact_stage: Self::expect_env_key("AIRSHPPER_ARTIFACT_STAGE"),
            target_branch: Self::expect_env_key("AIRSHIPPER_TARGET_BRANCH"),
            target_executable: Self::expect_env_key("AIRSHIPPER_TARGET_EXECUTABLES")
                .split(",")
                .map(|x| x.to_string())
                .collect(),
            bucket_name: Self::expect_env_key("AIRSHIPPER_BUCKET_NAME"),
            bucket_endpoint: Self::expect_env_key("AIRSHIPPER_BUCKET_ENDPOINT"),
            bucket_region: s3::region::Region::Custom {
                region: Self::expect_env_key("AIRSHIPPER_BUCKET_REGION"),
                endpoint: Self::expect_env_key("AIRSHIPPER_BUCKET_ENDPOINT"),
            },
            bucket_access_key: Self::expect_env_key("AIRSHIPPER_BUCKET_ACCESS_KEY"),
            bucket_secret_key: Self::expect_env_key("AIRSHIPPER_BUCKET_SECRET_KEY"),
            // Optional
            address: std::env::var("AIRSHIPPER_ADDRESS")
                .unwrap_or("0.0.0.0:8080".into())
                .parse()
                .unwrap_or("0.0.0.0:8080".parse().unwrap()),
            log_level: LogLevel::try_from(Self::get_env_key_or("AIRSHIPPER_LOG_LEVEL", "critical"))?,
            rocket_env: Env::try_from(Self::get_env_key_or("AIRSHIPPER_ROCKET_ENV", "production"))?,
        })
    }

    pub fn rocket(&self) -> Rocket {
        let config = Config::build(self.rocket_env.into())
            .address(self.address.ip().to_string())
            .port(self.address.port())
            .limits(Limits::default())
            .log_level(self.log_level.into());

        std::env::set_var("ROCKET_CLI_COLORS", "off");
        rocket::custom(config.finalize().expect("Invalid Config!"))
    }

    fn expect_env_key(name: &str) -> String {
        std::env::var(name).expect(&format!("required '{}' env key is not set!", name))
    }

    fn get_env_key_or(name: &str, unwrap_or: &str) -> String {
        std::env::var(name).unwrap_or(unwrap_or.into())
    }
}

/// Serializable and Deserializable LoggingLevel
#[derive(Copy, Clone, Debug, Display)]
pub enum LogLevel {
    Off,
    Normal,
    Debug,
    Critical,
}

/// Serializable and Deserializable Environment
#[derive(Copy, Clone, Debug, Display)]
pub enum Env {
    Development,
    Staging,
    Production,
}

impl TryFrom<String> for LogLevel {
    type Error = crate::ServerError;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "off" => Ok(LogLevel::Off),
            "normal" => Ok(LogLevel::Normal),
            "debug" => Ok(LogLevel::Debug),
            "critical" => Ok(LogLevel::Critical),
            x => Err(format!("Unknown LogLevel '{}'!", x).into()),
        }
    }
}

impl TryFrom<String> for Env {
    type Error = crate::ServerError;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "development" => Ok(Env::Development),
            "staging" => Ok(Env::Staging),
            "production" => Ok(Env::Production),
            x => Err(format!("Unknown Rocket Env '{}'!", x).into()),
        }
    }
}

impl Into<LoggingLevel> for LogLevel {
    fn into(self) -> LoggingLevel {
        match self {
            Self::Off => LoggingLevel::Off,
            Self::Normal => LoggingLevel::Normal,
            Self::Debug => LoggingLevel::Debug,
            Self::Critical => LoggingLevel::Critical,
        }
    }
}

impl Into<Environment> for Env {
    fn into(self) -> Environment {
        match self {
            Self::Development => Environment::Development,
            Self::Staging => Environment::Staging,
            Self::Production => Environment::Production,
        }
    }
}
