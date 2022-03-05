pub use crate::config::loading::{GithubReleaseConfig, Platform};
use regex::Regex;
use rocket::{serde::json::Value, Rocket};
use std::collections::HashMap;

pub mod loading;

/// The project ID of veloren on gitlab.
pub const PROJECT_ID: u64 = 10_174_980;
/// The Hook Type which gets parsed for artifacts.
pub const HOOK_TYPE: &str = "Pipeline Hook";

const DEFAULT_DATA_PATH: &str = "data";
pub const LOCAL_STORAGE_PATH: &str = "local";
const DATABASE_FILE: &str = "airshipper.db";
/// path: /opt/airshipper/config/config.ron
pub const CONFIG_PATH: &str = "config/config.ron";

#[derive(Debug, Clone)]
pub struct Config {
    pub channels: HashMap<String, Channel>,
    pub data_path: String,
}

#[derive(Clone)]
pub enum Filter {
    Stage { regex: Regex },
    TargetBranch { regex: Regex },
    BuildName { regex: Regex },
    //Environment { regex: String },
    //Variable { regex: String },
}

#[derive(Clone, Debug)]
pub struct AndFilter(pub Vec<Filter>);

#[derive(Clone, Debug)]
pub struct PlatformMapper {
    pub filter: AndFilter,
    pub platform: Platform,
}

#[derive(Clone)]
pub struct Channel {
    pub name: String,
    pub gitlab_secret: String,
    pub github_release_config: Option<GithubReleaseConfig>,
    pub channel_filters: Vec<AndFilter>,
    pub build_map: Vec<PlatformMapper>,
}

impl std::fmt::Debug for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::Stage { regex } => write!(f, "ArtifactStage({})", regex),
            Filter::TargetBranch { regex } => write!(f, "TargetBranch({})", regex),
            Filter::BuildName { regex } => write!(f, "BuildName({})", regex),
            //Filter::Environment { regex} => write!(f, "Environment({})", regex),
            //Filter::Variable { regex} => write!(f, "Variable({})", regex),
        }
    }
}

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Channel({})", self.name)
    }
}

impl Filter {
    pub(crate) fn compile(
        filter: loading::Filter,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(match filter {
            loading::Filter::Stage(regex) => Filter::Stage {
                regex: Regex::new(&regex)?,
            },
            loading::Filter::TargetBranch(regex) => Filter::TargetBranch {
                regex: Regex::new(&regex)?,
            },
            loading::Filter::BuildName(regex) => Filter::BuildName {
                regex: Regex::new(&regex)?,
            },
            //loading::Filter::Environment ( regex ) => Filter::Environment { regex:
            // Regex::new(&regex)? }, loading::Filter::Variable ( regex ) =>
            // Filter::Variable { regex: Regex::new(&regex)? },
        })
    }

    pub(crate) fn apply(
        &self,
        pipeline: &crate::models::PipelineUpdate,
        build_id: usize,
    ) -> bool {
        match self {
            Filter::Stage { regex } => match pipeline.builds.get(build_id) {
                Some(b) => regex.is_match(&b.stage),
                None => false,
            },
            Filter::TargetBranch { regex } => {
                regex.is_match(&pipeline.object_attributes.branch)
            },
            Filter::BuildName { regex } => match pipeline.builds.get(build_id) {
                Some(b) => regex.is_match(&b.name),
                None => false,
            },
            //Filter::Environment { regex} => write!(f, "Environment({})", regex),
            //Filter::Variable { regex} => write!(f, "Variable({})", regex),
        }
    }
}

impl AndFilter {
    pub(crate) fn compile(
        filter: loading::AndFilter,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut vec = vec![];
        for elem in filter.0 {
            vec.push(Filter::compile(elem)?);
        }
        Ok(AndFilter(vec))
    }

    pub(crate) fn apply(
        &self,
        pipeline: &crate::models::PipelineUpdate,
        build_id: usize,
    ) -> bool {
        for f in &self.0 {
            if !f.apply(pipeline, build_id) {
                return false;
            }
        }
        true
    }
}

impl PlatformMapper {
    pub fn compile(
        platform_matter: loading::PlatformMapper,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(PlatformMapper {
            filter: AndFilter::compile(platform_matter.filter)?,
            platform: platform_matter.platform,
        })
    }
}

impl Channel {
    pub fn compile(
        channel: loading::Channel,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut channel_filters = vec![];
        for elem in channel.channel_filters {
            channel_filters.push(AndFilter::compile(elem)?);
        }
        let mut build_map = vec![];
        for elem in channel.build_map {
            build_map.push(PlatformMapper::compile(elem)?);
        }
        Ok(Channel {
            name: channel.name,
            gitlab_secret: channel.gitlab_secret,
            github_release_config: channel.github_release_config,
            channel_filters,
            build_map,
        })
    }
}

impl Config {
    pub fn compile(config: loading::Config) -> Result<Self, Box<dyn std::error::Error>> {
        let mut channels = HashMap::new();
        for c in config.channels {
            channels.insert(c.name.clone(), Channel::compile(c)?);
        }
        Ok(Self {
            channels,
            data_path: config.data_path,
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ObjectAttributes, *};

    fn pipeline() -> PipelineUpdate {
        PipelineUpdate {
            object_kind: "pipeline".to_owned(),
            object_attributes: ObjectAttributes {
                id: 483899659,
                branch: "master".to_owned(),
                tag: false,
                sha: "d4c98e877501d80af663efa7601b5d36007f5593".to_owned(),
                before_sha: "31062c3761a0f37c4501abf998b8d86f9097acf9".to_owned(),
                status: "success".to_owned(),
                stages: vec!["build".to_owned(), "publish".to_owned(), "deploy".to_owned()],
                created_at: Some("2022-03-03 16:37:55 UTC".to_owned()),
                finished_at: Some("2022-03-03 17:21:30 UTC".to_owned()),
                duration: Some(2613),
                variables: vec!(),
            },
            user: User {
                name: "Marcel".to_owned(),
                username: "xMAC94x".to_owned(),
                avatar_url: "https://gitlab.com/uploads/-/system/user/avatar/276442/avatar.png".to_owned(),
            },
            project: Project {
                id: 10174980,
                name: "veloren".to_owned(),
                description: "Veloren is a multiplayer voxel RPG written in Rust. It is inspired by games such as Cube World, Legend of Zelda: Breath of the Wild, Dwarf Fortress and Minecraft.".to_owned(),
                web_url: "https://gitlab.com/veloren/veloren".to_owned(),
                avatar_url: Some("https://gitlab.com/uploads/-/system/project/avatar/10174980/veloren-square-big.png".to_owned()),
                git_ssh_url: "git@gitlab.com:veloren/veloren.git".to_owned(),
                git_http_url: "https://gitlab.com/veloren/veloren.git".to_owned(),
                namespace: "Veloren".to_owned(),
                visibility_level: 20,
                path_with_namespace: "veloren/veloren".to_owned(),
                default_branch: "master".to_owned(),
            },
            commit: Commit {
                id: "d4c98e877501d80af663efa7601b5d36007f5593".to_owned(),
                message: "Merge branch 'xMAC94x/linux-aarch64' into 'master'\n\nadd linux-aarch64 on master builds to be consistent\n\nSee merge request veloren/veloren!3250".to_owned(),
                timestamp: chrono::DateTime::from_utc(chrono::NaiveDate::from_ymd(2022, 3, 3).and_hms(16, 37, 53), chrono::Utc),
                url: "https://gitlab.com/veloren/veloren/-/commit/d4c98e877501d80af663efa7601b5d36007f5593".to_owned(),
                author: Author {
                    name: "Marcel".to_owned(),
                    email: "Marcel@example.com".to_owned(),
                }
            },
            builds: vec!(
                Build {
                    id: 2160133365,
                    stage: "build".to_owned(),
                    name: "linux-x86_64".to_owned(),
                    created_at: "2022-03-03 16:37:55 UTC".to_owned(),
                    started_at: Some("2022-03-03 16:37:57 UTC".to_owned()),
                    finished_at: Some("2022-03-03 17:02:50 UTC".to_owned()),
                    when: "on_success".to_owned(),
                    manual: false,
                    user: User {
                        name: "Marcel".to_owned(),
                        username: "xMAC94x".to_owned(),
                        avatar_url: "https://gitlab.com/uploads/-/system/user/avatar/276442/avatar.png".to_owned(),
                    },
                    runner: Some(Runner {
                        id: 276442,
                        description: "Marcel".to_owned(),
                        active: true,
                        is_shared: Some(false),
                    }),
                    artifacts_file: ArtifactsFile {
                        filename: Some("artifacts.zip".to_owned()),
                        size: Some(225124716),
                    },
                },
                Build {
                    id: 2160133363,
                    stage: "build".to_owned(),
                    name: "benchmarks".to_owned(),
                    created_at: "2022-03-03 16:37:55 UTC".to_owned(),
                    started_at: Some("2022-03-03 16:42:55 UTC".to_owned()),
                    finished_at: Some("2022-03-03 17:10:41 UTC".to_owned()),
                    when: "on_success".to_owned(),
                    manual: false,
                    user: User {
                        name: "Marcel".to_owned(),
                        username: "xMAC94x".to_owned(),
                        avatar_url: "https://gitlab.com/uploads/-/system/user/avatar/276442/avatar.png".to_owned(),
                    },
                    runner: Some(Runner {
                        id: 4236155,
                        description: "Rendezvous-epsilon".to_owned(),
                        active: true,
                        is_shared: Some(false),
                    }),
                    artifacts_file: ArtifactsFile {
                        filename: None,
                        size: None,
                    },
                },
                Build {
                    id: 2160133374,
                    stage: "publish".to_owned(),
                    name: "pages".to_owned(),
                    created_at: "2022-03-03 16:37:55 UTC".to_owned(),
                    started_at: Some("2022-03-03 17:10:41 UTC".to_owned()),
                    finished_at: Some("2022-03-03 17:21:30 UTC".to_owned()),
                    when: "on_success".to_owned(),
                    manual: false,
                    user: User {
                        name: "Marcel".to_owned(),
                        username: "xMAC94x".to_owned(),
                        avatar_url: "https://gitlab.com/uploads/-/system/user/avatar/276442/avatar.png".to_owned(),
                    },
                    runner: Some(Runner {
                        id: 4236155,
                        description: "Rendezvous-epsilon".to_owned(),
                        active: true,
                        is_shared: Some(false),
                    }),
                    artifacts_file: ArtifactsFile {
                        filename: Some("artifacts.zip".to_owned()),
                        size: Some(45127949),
                    },
                },
                Build {
                    id: 2160133371,
                    stage: "build".to_owned(),
                    name: "macos-aarch64".to_owned(),
                    created_at: "2022-03-03 16:37:55 UTC".to_owned(),
                    started_at: Some("2022-03-03 16:38:35 UTC".to_owned()),
                    finished_at: Some("2022-03-03 16:39:11 UTC".to_owned()),
                    when: "on_success".to_owned(),
                    manual: false,
                    user: User {
                        name: "Marcel".to_owned(),
                        username: "xMAC94x".to_owned(),
                        avatar_url: "https://gitlab.com/uploads/-/system/user/avatar/276442/avatar.png".to_owned(),
                    },
                    runner: Some(Runner {
                        id: 6565827,
                        description: "xvar-macos".to_owned(),
                        active: true,
                        is_shared: Some(false),
                    }),
                    artifacts_file: ArtifactsFile {
                        filename: Some("artifacts.zip".to_owned()),
                        size: Some(185762816),
                    },
                },
            ),
        }
    }

    #[test]
    fn filter_stage() {
        let p = pipeline();
        let filter = Filter::Stage {
            regex: Regex::new("build").unwrap(),
        };
        assert!(filter.apply(&p, 0));
        assert!(filter.apply(&p, 1));
        assert!(!filter.apply(&p, 2));
        assert!(filter.apply(&p, 3));
    }

    #[test]
    fn filter_stage_regex() {
        let p = pipeline();
        let filter = Filter::Stage {
            regex: Regex::new(".*ublis.*").unwrap(),
        };
        assert!(!filter.apply(&p, 0));
        assert!(!filter.apply(&p, 1));
        assert!(filter.apply(&p, 2));
        assert!(!filter.apply(&p, 3));
    }

    #[test]
    fn filter_target_branch() {
        let p = pipeline();
        let filter = Filter::TargetBranch {
            regex: Regex::new("master").unwrap(),
        };
        assert!(filter.apply(&p, 0));
        assert!(filter.apply(&p, 1));
        assert!(filter.apply(&p, 2));
        assert!(filter.apply(&p, 3));
    }

    #[test]
    fn filter_no_target_branch() {
        let p = pipeline();
        let filter = Filter::TargetBranch {
            regex: Regex::new("nightly").unwrap(),
        };
        assert!(!filter.apply(&p, 0));
        assert!(!filter.apply(&p, 1));
        assert!(!filter.apply(&p, 2));
        assert!(!filter.apply(&p, 3));
    }

    #[test]
    fn filter_build_linux() {
        let p = pipeline();
        let filter = Filter::BuildName {
            regex: Regex::new("linux-.*").unwrap(),
        };
        assert!(filter.apply(&p, 0));
        assert!(!filter.apply(&p, 1));
        assert!(!filter.apply(&p, 2));
        assert!(!filter.apply(&p, 3));
    }

    #[test]
    fn filter_build_aarch64() {
        let p = pipeline();
        let filter = Filter::BuildName {
            regex: Regex::new(".*-aarch64").unwrap(),
        };
        assert!(!filter.apply(&p, 0));
        assert!(!filter.apply(&p, 1));
        assert!(!filter.apply(&p, 2));
        assert!(filter.apply(&p, 3));
    }

    #[test]
    fn andfilter_build_macos() {
        let p = pipeline();
        let filter1 = Filter::BuildName {
            regex: Regex::new("macos-.*").unwrap(),
        };
        let filter2 = Filter::Stage {
            regex: Regex::new("build").unwrap(),
        };
        let filter = AndFilter(vec![filter1, filter2]);
        assert!(!filter.apply(&p, 0));
        assert!(!filter.apply(&p, 1));
        assert!(!filter.apply(&p, 2));
        assert!(filter.apply(&p, 3));
    }
}
