use derive_more::Display;
use rocket::config::*;
use rocket::Rocket;
use serde::{Deserialize, Serialize};

/// The project ID of veloren on gitlab.
pub const PROJECT_ID: u64 = 10174980;
/// The Hook Type which gets parsed for artifacts.
pub const HOOK_TYPE: &str = "Pipeline Hook";
/// Location of the config
pub const CONFIG_PATH: &str = "server.ron";

/// File ending of *downloaded* artifacts
#[cfg(not(unix))]
pub const ARTIFACT_FILE_ENDING: &str = "zip";
#[cfg(unix)]
pub const ARTIFACT_FILE_ENDING: &str = "tar.gz";

/// Configuration and defaults for the entire server.
/// Provides configuration for essentials like port, addr and secrets
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub address: std::net::SocketAddr,
    pub database_address: String,
    pub database_table: String,
    pub static_files: String,
    
    /// Off, Normal, Debug, Critical
    pub log_level: LogLevel,
    /// Development, Staging, Production
    pub rocket_env: Env,
    pub log_colors: bool,

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

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:8080".parse().unwrap(),
            database_address: "postgres://username:password@localhost:5432".into(),
            database_table: "artifacts".into(),
            static_files: "public/".into(),
            log_level: LogLevel::Critical,
            rocket_env: Env::Production,
            log_colors: true,

            gitlab_secret: "".into(),
            artifact_stage: "".into(),
            target_branch: "master".into(),
            target_executable: Vec::new(),
        }
    }
}

impl ServerConfig {
    pub fn load() -> Self {
        // Return test config if running tests
        #[cfg(feature = "test")]
        {
            println!("Initialising with test configuration");
            return Self::test_config();
        }

        // Load up from disk
        #[cfg(not(feature = "test"))]
        {
            if let Ok(file) = std::fs::File::open(CONFIG_PATH) {
                match ron::de::from_reader(file) {
                    Ok(c) => return c,
                    Err(_) => {
                        // Failed parsing
                        println!("WARNING: Failed parsing config. Reset to default.");
                    }
                }
            }

            // File does not exist.
            println!("NOTE: Creating default config. Please configure...");
            let default = Self::default();
            default.save_to_file();
            println!("Exiting...");
            std::process::exit(0);
        }
    }

    pub fn rocket(&self) -> Rocket {
        use std::collections::HashMap;
        // Prepare public folder
        std::fs::create_dir_all(&self.static_files).expect("Failed to prepare public folder!");

        // Set database url
        let mut database_config = HashMap::new();
        let mut databases = HashMap::new();
        database_config.insert("url", Value::from(self.database_address.clone()));
        databases.insert("postgres", Value::from(database_config));

        let config = Config::build(self.rocket_env.into())
            .address(self.address.ip().to_string())
            .port(self.address.port())
            .limits(Limits::default())
            .log_level(self.log_level.into())
            .extra("databases", databases);

        if !self.log_colors {
            std::env::set_var("ROCKET_CLI_COLORS", "off");
        }
        rocket::custom(config.finalize().expect("Invalid Config!"))
    }

    #[cfg(not(feature = "test"))]
    fn save_to_file(&self) {
        use std::io::Write;

        let mut config_file =
            std::fs::File::create(CONFIG_PATH).expect("Failed to create config file!");

        let serialised: &str = &ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .expect("Failed to serialise config!");
        config_file
            .write_all(serialised.as_bytes())
            .expect("failed writing config!");
    }

    /// Returns a configuration suited for testing
    #[cfg(feature = "test")]
    fn test_config() -> ServerConfig {
        let mut base = Self::default();
        base.gitlab_secret = "gitlab_secret_test".into();
        base.database_address = "postgres://postgres:postgres@localhost:5432".into();
        base.database_table = "artifacts_test".into(); // TODO: Populate with dummy values!
        base.log_colors = false;
        base.log_level = LogLevel::Debug;
        base.rocket_env = Env::Development;
        base.static_files = "tests/webhook/public".into();
        base.artifact_stage = "post".into();
        base.target_executable = vec!["windows".into(), "linux".into()];

        base
    }
}

/// Serializable and Deserializable LoggingLevel
#[derive(Copy, Clone, Debug, Display, Serialize, Deserialize)]
pub enum LogLevel {
    Off,
    Normal,
    Debug,
    Critical,
}

/// Serializable and Deserializable Environment
#[derive(Copy, Clone, Debug, Display, Serialize, Deserialize)]
pub enum Env {
    Development,
    Staging,
    Production,
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
