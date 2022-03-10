use crate::{
    config::{Channel, Platform},
    db::{schema::artifacts, DbArtifact, FsStorage},
    models::{Build, PipelineUpdate},
};
use chrono::NaiveDateTime;
use diesel::Queryable;
use url::Url;

#[derive(Debug, Queryable, Insertable, Clone)]
#[table_name = "artifacts"]
pub struct Artifact {
    pub build_id: i64,
    pub date: NaiveDateTime,
    pub hash: String,
    pub author: String,
    pub merged_by: String,

    pub os: String,
    pub arch: String,
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
            os: db.os.clone(),
            arch: db.arch.clone(),
            channel: db.channel.clone(),
            file_name: db.file_name.clone(),
            download_uri: db.download_uri.clone(),
        }
    }
}

impl Artifact {
    pub fn try_from(
        pipe: &PipelineUpdate,
        channel: &Channel,
        build: &Build,
        platform: &Platform,
    ) -> Option<Self> {
        // Check if it contains artifact
        if build.artifacts_file.filename.is_some() {
            // Datetime<Utc> isn't store-able in diesel.
            let date = pipe.commit.timestamp.naive_utc();

            let build_id = build.id as i64;
            let os = platform.os.clone();
            let arch = platform.arch.clone();
            let platform = platform.to_string();
            let file_name = format!(
                "{}-{}-{}.zip",
                &channel.name,
                platform,
                date.format("%Y-%m-%dT%H_%M")
            );
            let download_uri = format!("/{}", FsStorage::get_download_url(&file_name));

            Some(Self {
                build_id,
                date,
                hash: pipe.object_attributes.sha.clone(),
                author: pipe.commit.author.name.clone(),
                merged_by: pipe.user.name.clone(),
                os,
                arch,
                channel: channel.name.clone(),
                file_name,
                download_uri,
            })
        } else {
            None
        }
    }

    pub fn get_artifact_url(&self) -> Url {
        Url::parse(&format!(
            "https://gitlab.com/api/v4/projects/{}/jobs/{}/artifacts",
            crate::config::PROJECT_ID,
            self.build_id
        ))
        .unwrap()
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
}
