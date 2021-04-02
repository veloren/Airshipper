use super::Artifact;
use crate::CONFIG;
use chrono::{DateTime, Utc};
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
    pub(crate) fn artifacts(&self) -> Option<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        if self.object_attributes.branch != CONFIG.target_branch {
            tracing::debug!(
                "Branch '{}' does not match target '{}'",
                self.object_attributes.branch,
                CONFIG.target_branch
            );
            return None;
        }

        if let Some(target_variable) = &CONFIG.target_variable {
            if !self
                .object_attributes
                .variables
                .iter()
                .any(|e| &e.key == target_variable)
            {
                tracing::debug!("Variable '{}' was not found", target_variable);
                return None;
            }
        }

        for build in &self.builds {
            // Skip non-artifact builds.
            if build.stage != crate::CONFIG.artifact_stage {
                tracing::debug!("Skipping artifact in '{}' stage...", build.stage);
                continue;
            }

            if let Some(artifact) = Artifact::try_from(&self, build) {
                artifacts.push(artifact);
            }
        }

        if artifacts.is_empty() { None } else { Some(artifacts) }
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

#[derive(Debug, Clone, Deserialize)]
pub struct Runner {
    pub id: u64,
    pub description: String,
    pub active: bool,
    pub is_shared: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactsFile {
    pub filename: Option<String>,
    pub size: Option<u64>,
}
