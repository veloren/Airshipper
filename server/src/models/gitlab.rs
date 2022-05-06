use super::Artifact;
use crate::CONFIG;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PipelineUpdate {
    pub object_kind: String,
    pub object_attributes: ObjectAttributes,
    pub user: User,
    pub project: Project,
    pub commit: Commit,
    pub builds: Vec<Build>,
}

impl PipelineUpdate {
    ///Global Filter for invalid webhooks
    pub(crate) fn early_filter(&self) -> bool {
        if self.object_attributes.status != "success" {
            let status = &self.object_attributes.status;
            tracing::debug!(?status, "Skipping update as it isn't successful",);
            return false;
        }
        true
    }

    pub(crate) fn channel(&self) -> Option<String> {
        for (channel_name, channel) in &crate::CONFIG.channels {
            // check if at least one filter matches
            for filter in &channel.channel_filters {
                let _guard = tracing::info_span!("channel-filter", ?channel, ?filter);
                tracing::trace!("checking");
                // The filter must apply to at least 1 build_id
                for (build, _) in self.builds.iter().enumerate() {
                    match filter.apply(self, build) {
                        Ok(()) => {
                            tracing::debug!(
                                ?build,
                                "Filter applied successful, channel determined",
                            );
                            return Some(channel_name.clone());
                        },
                        Err(e) => tracing::trace!(
                            ?build,
                            ?e,
                            "Filter returned error, channel does not match",
                        ),
                    }
                }
            }
        }
        None
    }

    pub(crate) fn artifacts(&self, channel: &str) -> Vec<Artifact> {
        let mut artifacts = Vec::new();

        let channel = CONFIG.channels.get(channel).unwrap();

        // Apply filters
        for (i, build) in self.builds.iter().enumerate() {
            // find matching Platform
            for filter in &channel.build_map {
                match filter.filter.apply(self, i) {
                    Ok(()) => {
                        let platform = &filter.platform;
                        let filter = &filter.filter;
                        if let Some(artifact) =
                            Artifact::try_from(self, channel, build, platform)
                        {
                            let id = artifact.build_id;
                            tracing::trace!(?id, ?filter, "artifact matched with");
                            artifacts.push(artifact);
                        }
                    },
                    Err(e) => tracing::trace!(
                        ?filter,
                        ?build.name,
                        ?e,
                        "Filter returned error, build didn't add as artfact",
                    ),
                }
            }
        }

        artifacts
    }

    pub(crate) async fn extends_variables(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Debug, Deserialize)]
        struct LastPipeline {
            id: u64,
        }

        #[derive(Debug, Deserialize)]
        struct Schedule {
            //id: u64,
            variables: Vec<Variable>,
            last_pipeline: LastPipeline,
        }

        #[derive(Debug, Deserialize)]
        struct Schedules {
            id: u64,
        }

        if let Some(token) = &crate::CONFIG.gitlab_token {
            let pipeline_id = self.object_attributes.id;
            // get all schedules available
            let mut headers = HeaderMap::new();
            headers.insert("PRIVATE-TOKEN", HeaderValue::from_str(token)?);
            headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
            let client = reqwest::Client::builder()
                .default_headers(headers)
                .build()?;
            let schedules = client
                .get(format!(
                    "https://gitlab.com/api/v4/projects/{}/pipeline_schedules",
                    crate::config::PROJECT_ID,
                ))
                .send()
                .await?
                .json::<Vec<Schedules>>()
                .await?;
            tracing::trace!(?schedules, "Schedules");
            for schedule in schedules {
                let id = schedule.id;
                let mut details = client
                    .get(format!(
                        "https://gitlab.com/api/v4/projects/{}/pipeline_schedules/{}",
                        crate::config::PROJECT_ID,
                        id
                    ))
                    .send()
                    .await?
                    .json::<Schedule>()
                    .await?;
                tracing::trace!(?details, "Details");
                if details.last_pipeline.id == pipeline_id {
                    self.object_attributes
                        .variables
                        .append(&mut details.variables);
                }
            }
        } else {
            tracing::trace!("skip variable extension, no gitlab token provided");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Variable {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectAttributes {
    pub id: u64,
    #[serde(rename = "ref")]
    pub branch: String,
    pub tag: bool,
    pub sha: String,
    pub before_sha: String,
    pub status: String,
    pub stages: Vec<String>,
    pub created_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration: Option<u64>,
    pub variables: Vec<Variable>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub name: String,
    pub username: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub web_url: String,
    pub avatar_url: Option<String>,
    pub git_ssh_url: String,
    pub git_http_url: String,
    pub namespace: String,
    pub visibility_level: u64,
    pub path_with_namespace: String,
    pub default_branch: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub url: String,
    pub author: Author,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Build {
    pub id: u64,
    pub stage: String,
    pub name: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub when: String,
    pub manual: bool,
    pub user: User,
    pub runner: Option<Runner>,
    pub artifacts_file: ArtifactsFile,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Runner {
    pub id: u64,
    pub description: String,
    pub active: bool,
    pub is_shared: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactsFile {
    pub filename: Option<String>,
    pub size: Option<u64>,
}
