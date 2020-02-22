use derive_more::Display;
use rocket::config::*;
use rocket::Rocket;
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
    pub log_colors: bool,
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
            gitlab_secret: std::env::var("AIRSHIPPER_GITLAB_SECRET").expect("AIRSHIPPER_GITLAB_SECRET not set!"),
            artifact_stage: std::env::var("AIRSHPPER_ARTIFACT_STAGE").expect("AIRSHPPER_ARTIFACT_STAGE not set!"),
            target_branch: std::env::var("AIRSHIPPER_TARGET_BRANCH").expect("AIRSHIPPER_TARGET_BRANCH not set!"),
            target_executable: std::env::var("AIRSHIPPER_TARGET_EXECUTABLES").expect("AIRSHIPPER_TARGET_EXECUTABLES not set!")
                .split(",")
                .map(|x| x.to_string())
                .collect(),
            bucket_name: std::env::var("AIRSHIPPER_BUCKET_NAME").expect("AIRSHIPPER_BUCKET_NAME not set!"),
            bucket_endpoint: std::env::var("AIRSHIPPER_BUCKET_ENDPOINT").expect("AIRSHIPPER_BUCKET_ENDPOINT not set!"),
            bucket_region: s3::region::Region::Custom {
                region: std::env::var("AIRSHIPPER_BUCKET_REGION").expect("AIRSHIPPER_BUCKET_REGION not set!"),
                endpoint: std::env::var("AIRSHIPPER_BUCKET_ENDPOINT").expect("AIRSHIPPER_BUCKET_ENDPOINT not set!"),
            },
            bucket_access_key: std::env::var("AIRSHIPPER_BUCKET_ACCESS_KEY").expect("AIRSHIPPER_BUCKET_ACCESS_KEY not set!"),
            bucket_secret_key: std::env::var("AIRSHIPPER_BUCKET_SECRET_KEY").expect("AIRSHIPPER_BUCKET_SECRET_KEY not set!"),
            // Optional
            address: std::env::var("AIRSHIPPER_ADDRESS").unwrap_or("0.0.0.0:8080".into())
                .parse()
                .unwrap_or("0.0.0.0:8080".parse().unwrap()),
            log_level: LogLevel::try_from(std::env::var("AIRSHIPPER_LOG_LEVEL").unwrap_or("critical".into()))
                .unwrap_or(LogLevel::Critical),
            rocket_env: Env::try_from(std::env::var("AIRSHIPPER_ROCKET_ENV").unwrap_or("production".into()))
                .unwrap_or(Env::Production),
            log_colors: std::env::var("AIRSHIPPER_COLORS").unwrap_or("false".into()).parse().unwrap_or(false),
        })
    }

    pub fn rocket(&self) -> Rocket {
        let config = Config::build(self.rocket_env.into())
            .address(self.address.ip().to_string())
            .port(self.address.port())
            .limits(Limits::default())
            .log_level(self.log_level.into());

        if !self.log_colors {
            std::env::set_var("ROCKET_CLI_COLORS", "off");
        }
        rocket::custom(config.finalize().expect("Invalid Config!"))
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
            "Debug" => Ok(LogLevel::Debug),
            "Critical" => Ok(LogLevel::Critical),
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
