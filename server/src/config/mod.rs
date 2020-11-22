use rocket::{config::*, Rocket};

/// The project ID of veloren on gitlab.
pub const PROJECT_ID: u64 = 10_174_980;
/// The Hook Type which gets parsed for artifacts.
pub const HOOK_TYPE: &str = "Pipeline Hook";

pub const DATABASE_FILE: &str = "airshipper.db";

/// Configuration and defaults for the entire server.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// DO Space details
    pub bucket_name: String,
    pub bucket_region: String,
    pub bucket_endpoint: String,
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

    /// Whether to use DigitalOcean Spaces CDN.
    pub spaces_cdn: bool,
}

impl ServerConfig {
    pub fn load() -> Self {
        let cfg = Self {
            bucket_name: Self::expect_env_key("AIRSHIPPER_BUCKET_NAME"),
            bucket_endpoint: Self::expect_env_key("AIRSHIPPER_BUCKET_ENDPOINT"),
            bucket_region: Self::expect_env_key("AIRSHIPPER_BUCKET_REGION"),
            bucket_access_key: Self::expect_env_key("AIRSHIPPER_BUCKET_ACCESS_KEY"),
            bucket_secret_key: Self::expect_env_key("AIRSHIPPER_BUCKET_SECRET_KEY"),

            gitlab_secret: Self::expect_env_key("AIRSHIPPER_GITLAB_SECRET"),
            artifact_stage: Self::expect_env_key("AIRSHIPPER_ARTIFACT_STAGE"),
            target_executable: Self::expect_env_key("AIRSHIPPER_TARGET_EXECUTABLES")
                .split(',')
                .map(|x| x.to_string())
                .collect(),
            spaces_cdn: Self::expect_env_key("AIRSHIPPER_SPACES_CDN").parse().unwrap_or(true),
            // Optional
            target_branch: Self::get_env_key_or("AIRSHIPPER_TARGET_BRANCH", "master"),
        };

        if cfg.spaces_cdn {
            tracing::info!("DigitalOcean Spaces CDN is enabled.");
        } else {
            tracing::info!("DigitalOcean Spaces CDN is DISABLED.");
        }

        cfg
    }

    pub fn rocket(&self) -> Rocket {
        use std::collections::HashMap;
        // Set database url
        let mut database_config = HashMap::new();
        let mut databases = HashMap::new();
        database_config.insert("url", Value::from(DATABASE_FILE));
        databases.insert("sqlite", Value::from(database_config));

        let config =
            Config::build(rocket::config::Environment::active().unwrap_or(rocket::config::Environment::Production))
                .extra("databases", databases);
        rocket::custom(config.finalize().expect("Invalid Config!"))
    }

    fn expect_env_key(name: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| panic!("required '{}' env key is not set!", name))
    }

    fn get_env_key_or(name: &str, unwrap_or: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| unwrap_or.into())
    }
}
