use rocket::{config::*, Rocket};

/// The project ID of veloren on gitlab.
pub const PROJECT_ID: u64 = 10_174_980;
/// The Hook Type which gets parsed for artifacts.
pub const HOOK_TYPE: &str = "Pipeline Hook";

const DEFAULT_DATA_PATH: &str = "data";
pub const LOCAL_STORAGE_PATH: &str = "local";
const DATABASE_FILE: &str = "airshipper.db";

/// Configuration and defaults for the entire server.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Specified secret to verify webhook is from gitlab
    pub gitlab_secret: String,
    /// At which stage of the pipeline the artifacts are uploaded.
    pub artifact_stage: String,
    /// What branch should be downloaded
    pub target_branch: String,
    /// What binary builds should be downloaded
    /// NOTE: These names have to include the OS!
    pub target_executable: Vec<String>,

    /// Path to the data directory
    pub data_path: String,
}

impl ServerConfig {
    pub fn load() -> Self {
        let cfg = Self {
            gitlab_secret: Self::expect_env_key("AIRSHIPPER_GITLAB_SECRET"),
            artifact_stage: Self::expect_env_key("AIRSHIPPER_ARTIFACT_STAGE"),
            target_executable: Self::expect_env_key("AIRSHIPPER_TARGET_EXECUTABLES")
                .split(',')
                .map(|x| x.to_string())
                .collect(),
            // Optional
            target_branch: Self::get_env_key_or("AIRSHIPPER_TARGET_BRANCH", "master"),
            data_path: Self::get_env_key_or("AIRSHIPPER_DATA_PATH", DEFAULT_DATA_PATH),
        };

        cfg
    }

    pub fn get_db_file_path(&self) -> std::path::PathBuf {
        let mut path = std::path::PathBuf::from(self.data_path.clone());
        path.push(DATABASE_FILE);
        path
    }

    pub fn get_local_storage_path(&self) -> std::path::PathBuf {
        let mut path = std::path::PathBuf::from(self.data_path.clone());
        path.push(LOCAL_STORAGE_PATH);
        path
    }

    pub fn rocket(&self) -> Rocket {
        use std::collections::HashMap;
        // Set database url
        let mut database_config = HashMap::new();
        let mut databases = HashMap::new();
        let dbpath = self.get_db_file_path();
        database_config.insert(
            "url",
            Value::from(dbpath.to_str().expect("non UTF8-path provided")),
        );
        databases.insert("sqlite", Value::from(database_config));

        let config = Config::build(
            rocket::config::Environment::active()
                .unwrap_or(rocket::config::Environment::Production),
        )
        .extra("databases", databases);
        rocket::custom(config.finalize().expect("Invalid Config!"))
    }

    fn expect_env_key(name: &str) -> String {
        std::env::var(name)
            .unwrap_or_else(|_| panic!("required '{}' env key is not set!", name))
    }

    fn get_env_key_or(name: &str, unwrap_or: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| unwrap_or.into())
    }
}
