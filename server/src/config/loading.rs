use super::DEFAULT_DATA_PATH;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use tracing::{error, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub channels: Vec<Channel>,
    /// Path to the data directory
    pub data_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Filter {
    Stage(String),
    TargetBranch(String),
    BuildName(String),
    //Environment ( String ),
    //Variable ( String ),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AndFilter(pub Vec<Filter>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Platform {
    pub os: String,
    pub arch: String,
}

/// Filter that need to apply to result in an platform configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformMapper {
    pub filter: AndFilter,
    pub platform: Platform,
}

/// Configuration for github releases.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GithubReleaseConfig {
    /// Github personal access token
    pub github_token: String,
    /// The user/group that owns the repository.
    pub github_repository_owner: String,
    /// The github repository name.
    pub github_repository: String,
    /// The tag name of the github release.
    pub github_release: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// public name under which this channel is known for.
    pub name: String,
    /// Specified secret to verify webhook is from gitlab
    pub gitlab_secret: String,
    /// Github release information
    pub github_release_config: Option<GithubReleaseConfig>,
    /// Filters that apply to determine if a webhook matches this channel
    pub channel_filters: Vec<AndFilter>,
    /// A list of Filters, the first Filter that matches determines the respective
    /// Platform that is used
    pub build_map: Vec<PlatformMapper>,
}

impl Default for Config {
    fn default() -> Self {
        let channel = Channel {
            name: "nightly".to_owned(),
            gitlab_secret: "secret".to_owned(),
            github_release_config: Some(GithubReleaseConfig {
                github_token: "token".to_owned(),
                github_repository_owner: "veloren".to_owned(),
                github_repository: "veloren".to_owned(),
                github_release: "test".to_owned(),
            }),
            channel_filters: vec![AndFilter(vec![Filter::TargetBranch(
                ".*master.*".to_owned(),
            )])],
            build_map: vec![PlatformMapper {
                filter: AndFilter(vec![Filter::BuildName(".*linux-x86_64.*".to_owned())]),
                platform: Platform {
                    os: "linux".to_owned(),
                    arch: "x86_64".to_owned(),
                },
            }],
        };
        Self {
            channels: vec![channel],
            data_path: DEFAULT_DATA_PATH.to_owned(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = fs::File::open(&path)?;
        match ron::de::from_reader(file) {
            Ok(x) => Ok(x),
            Err(e) => {
                let default_settings = Self::default();
                let template_path = path.with_extension("template.ron");
                warn!(
                    ?e,
                    "Failed to parse setting file! Falling back to default settings and \
                     creating a template file for you to migrate your current settings \
                     file: {}",
                    template_path.display()
                );
                if let Err(e) = default_settings.save_to_file(&template_path) {
                    error!(?e, "Failed to create template settings file")
                }
                Err(e.into())
            },
        }
    }

    fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        // Create dir if it doesn't exist
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let ron = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .expect("Failed serialize settings.");

        fs::write(path, ron.as_bytes())?;

        Ok(())
    }
}
