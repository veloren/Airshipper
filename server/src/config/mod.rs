use rocket::{config::*, Rocket};

/// The project ID of veloren on gitlab.
pub const PROJECT_ID: u64 = 10_174_980;
/// The Hook Type which gets parsed for artifacts.
pub const HOOK_TYPE: &str = "Pipeline Hook";

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

    /// Path to the database
    pub db_path: String,
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
            db_path: Self::get_env_key_or("AIRSHIPPER_DB_PATH", DATABASE_FILE),
        };

        cfg
    }

    pub fn rocket(&self) -> Rocket {
        use std::collections::HashMap;
        // Set database url
        let mut database_config = HashMap::new();
        let mut databases = HashMap::new();
        database_config.insert("url", Value::from(self.db_path.as_ref()));
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
