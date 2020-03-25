use crate::models::{Build, PipelineUpdate};
use chrono::NaiveDateTime;
use derive_more::Display;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Artifact {
    pub id: u64,
    pub date: NaiveDateTime,
    pub hash: String,
    pub author: String,
    pub merged_by: String,

    pub platform: Platform,
    pub channel: Channel,
    pub download_path: PathBuf,
    pub file_ending: String,
}

#[derive(Debug, Display, Clone, Copy)]
pub enum Platform {
    Windows,
    Linux,
}

#[derive(Debug, Display, Clone, Copy)]
pub enum Channel {
    Nightly,
    // TODO: Release,
}

impl Artifact {
    pub fn try_from(pipe: &PipelineUpdate, build: &Build) -> Option<Self> {
        // Check if it contains artifact
        if crate::CONFIG.target_executable.contains(&build.name) && build.artifacts_file.filename.is_some() {
            let date = NaiveDateTime::parse_from_str(
                &pipe.commit.timestamp.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                "%Y-%m-%dT%H:%M:%SZ",
            )
            .expect("Failed to parse date!");
            let id = build.id;
            let platform = Self::get_platform(&build.name)?;
            let channel = Self::get_channel();
            let file_ending = std::path::Path::new(&build.artifacts_file.filename.clone().unwrap()) // Unwrap fine. See above.
                .extension()
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or("zip".into());
            let download_path = Self::get_download_path(&date, &platform, &channel, &file_ending);

            Some(Self {
                id,
                date,
                hash: pipe.object_attributes.sha.clone(),
                author: pipe.commit.author.name.clone(),
                merged_by: pipe.user.name.clone(),
                platform,
                channel,
                download_path,
                file_ending,
            })
        } else {
            None
        }
    }

    pub fn get_url(&self) -> String {
        format!(
            "https://gitlab.com/api/v4/projects/{}/jobs/{}/artifacts",
            crate::config::PROJECT_ID,
            self.id
        )
    }

    pub fn get_download_path(
        date: &NaiveDateTime,
        platform: &Platform,
        channel: &Channel,
        file_ending: &String,
    ) -> PathBuf {
        PathBuf::new().join(format!(
            "{}-{}-{}.{}",
            channel,
            platform,
            date.format("%Y-%m-%d-%H_%M"),
            file_ending
        ))
    }

    fn get_platform(name: &str) -> Option<Platform> {
        if name.contains("windows") {
            Some(Platform::Windows)
        } else if name.contains("linux") {
            Some(Platform::Linux)
        } else {
            None
        }
    }

    fn get_channel() -> Channel {
        Channel::Nightly
    }
}
