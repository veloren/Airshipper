use crate::error::ServerError;
use crate::models::{Build, PipelineUpdate};
use crate::Result;
use chrono::NaiveDateTime;
use derive_more::Display;
use std::fs::File;
use std::io;
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
    pub fn try_from(pipe: &PipelineUpdate, build: &Build) -> Result<Option<Self>> {
        // Check if it contains artifact
        if crate::CONFIG.target_executable.contains(&build.name)
            && build.artifacts_file.filename.is_some()
        {
            // Ex: 2019-10-18T16:21:28Z
            // TODO: Find a better way to convert it...
            let date = NaiveDateTime::parse_from_str(
                &pipe
                    .commit
                    .timestamp
                    .format("%Y-%m-%dT%H:%M:%SZ")
                    .to_string(),
                "%Y-%m-%dT%H:%M:%SZ",
            )?;
            let id = build.id;
            let platform = Self::get_platform(&build.name)?;
            let channel = Self::get_channel();
            let download_path = Self::get_download_path(&date, &platform, &channel)?;

            Ok(Some(Self {
                id,
                date,
                hash: pipe.object_attributes.sha.clone(),
                author: pipe.commit.author.name.clone(),
                merged_by: pipe.user.name.clone(),
                platform,
                channel,
                download_path,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn download(&self) -> Result<()> {
        let mut req = reqwest::get(&self.get_url())?;
        if req.status().is_success() {
            let mut f = File::create(&self.download_path)?;
            io::copy(&mut req, &mut f)?;
        }
        Ok(())
    }

    fn get_download_path(
        date: &NaiveDateTime,
        platform: &Platform,
        channel: &Channel,
    ) -> Result<PathBuf> {
        let path = PathBuf::from(&crate::CONFIG.static_files).join(format!("{}/", platform));
        // Create base path
        std::fs::create_dir_all(&path)?;
        // Add file name + extension
        Ok(path.join(format!(
            "{}-{}.{}",
            channel,
            date.format("%Y-%m-%d-%H_%M_%S"),
            crate::config::ARTIFACT_FILE_ENDING,
        )))
    }

    fn get_url(&self) -> String {
        format!(
            "https://gitlab.com/api/v4/projects/{}/jobs/{}/artifacts",
            crate::config::PROJECT_ID,
            self.id
        )
    }

    fn get_platform(name: &str) -> Result<Platform> {
        if name.contains("windows") {
            Ok(Platform::Windows)
        } else if name.contains("linux") {
            Ok(Platform::Linux)
        } else {
            Err(ServerError::InvalidPlatform)
        }
    }

    fn get_channel() -> Channel {
        Channel::Nightly
    }
}
