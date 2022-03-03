use rocket::{serde::json::Value, Rocket};

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
    /// The user/group that owns the repository.
    pub github_repository_owner: String,
    /// The github repository name.
    pub github_repository: String,
    /// The tag name of the github release.
    pub github_release: String,
    /// Github personal access token
    pub github_token: String,
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
            github_token: Self::expect_env_key("AIRSHIPPER_GITHUB_TOKEN"),
            github_repository_owner: Self::expect_env_key(
                "AIRSHIPPER_GITHUB_REPOSITORY_OWNER",
            ),
            github_repository: Self::expect_env_key("AIRSHIPPER_GITHUB_REPOSITORY"),
            github_release: Self::expect_env_key("AIRSHIPPER_GITHUB_RELEASE"),
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

    pub fn rocket(&self) -> Rocket<rocket::Build> {
        use rocket::figment::{util::map, Figment};
        // Set database url
        let dbpath = self.get_db_file_path();
        let options =
            map!["url" => Value::from(dbpath.to_str().expect("non UTF8-path provided"))];
        let mut config = rocket::Config::release_default();
        config.log_level = rocket::log::LogLevel::Debug;
        config.address = std::net::Ipv4Addr::new(0, 0, 0, 0).into();

        let provider =
            Figment::from(config).merge(("databases", map!["sqlite" => &options]));

        rocket::custom(provider)
    }

    fn expect_env_key(name: &str) -> String {
        std::env::var(name)
            .unwrap_or_else(|_| panic!("required '{}' env key is not set!", name))
    }

    fn get_env_key_or(name: &str, unwrap_or: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| unwrap_or.into())
    }
}
