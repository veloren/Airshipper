use crate::{
    db::{schema::artifacts, DbArtifact},
    models::{Build, PipelineUpdate},
    CONFIG,
};
use chrono::NaiveDateTime;
use diesel::Queryable;

#[derive(Debug, Queryable, Insertable)]
#[table_name = "artifacts"]
pub struct Artifact {
    pub build_id: i32,
    pub date: NaiveDateTime,
    pub hash: String,
    pub author: String,
    pub merged_by: String,

    pub platform: String,
    pub channel: String,
    pub file_name: String,
    pub download_uri: String,
}

impl From<&DbArtifact> for Artifact {
    fn from(db: &DbArtifact) -> Self {
        Self {
            build_id: db.build_id,
            date: db.date,
            hash: db.hash.clone(),
            author: db.author.clone(),
            merged_by: db.merged_by.clone(),
            platform: db.platform.clone(),
            channel: db.channel.clone(),
            file_name: db.file_name.clone(),
            download_uri: db.download_uri.clone(),
        }
    }
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
            let build_id = build.id as i32;
            let platform = Self::get_platform(&build.name)?;
            let channel = Self::get_channel();
            let file_name = format!("{}-{}-{}.zip", channel, platform, date.format("%Y-%m-%d-%H_%M"));
            let download_uri = format!(
                "https://{}.{}.cdn.{}/nightly/{}",
                CONFIG.bucket_name, CONFIG.bucket_region, CONFIG.bucket_endpoint, file_name
            );

            Some(Self {
                build_id,
                date,
                hash: pipe.object_attributes.sha.clone(),
                author: pipe.commit.author.name.clone(),
                merged_by: pipe.user.name.clone(),
                platform,
                channel,
                file_name,
                download_uri,
            })
        } else {
            None
        }
    }

    pub fn get_url(&self) -> String {
        format!(
            "https://gitlab.com/api/v4/projects/{}/jobs/{}/artifacts",
            crate::config::PROJECT_ID,
            self.build_id
        )
    }

    pub fn get_download_url(filename: &str) -> String {
        match CONFIG.spaces_cdn {
            true => format!(
                "https://{}.{}.cdn.{}/nightly/{}",
                CONFIG.bucket_name, CONFIG.bucket_region, CONFIG.bucket_endpoint, filename
            ),
            false => format!(
                "https://{}.{}.{}/nightly/{}",
                CONFIG.bucket_name, CONFIG.bucket_region, CONFIG.bucket_endpoint, filename
            ),
        }
    }

    /// Returns the file extension
    /// NOTE: without dot (e.g. zip)
    pub fn extension(&self) -> String {
        use std::{ffi::OsStr, path::PathBuf};
        PathBuf::from(&self.file_name)
            .extension()
            .unwrap_or_else(|| OsStr::new("zip"))
            .to_string_lossy()
            .into()
    }

    fn get_platform(name: &str) -> Option<String> {
        if name.contains("windows") {
            Some("windows".into())
        } else if name.contains("linux") {
            Some("linux".into())
        } else if name.contains("macos") {
            Some("macos".into())
        } else {
            None
        }
    }

    fn get_channel() -> String {
        "nightly".into()
    }
}
