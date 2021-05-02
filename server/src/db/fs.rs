use crate::{models::v1::Artifact, Result};
use std::path::PathBuf;

#[derive(Default, Clone, Copy)]
pub struct FsStorage;

pub const ROOT_FOLDER: &str = "nightly";

impl FsStorage {
    /// store artifact
    #[tracing::instrument]
    pub async fn store(artifact: &Artifact) -> Result<()> {
        Self::store_file(&artifact.file_name).await?;
        Ok(())
    }

    /// Deletes artifact from filesystem storage.
    #[tracing::instrument]
    pub async fn delete(artifact: &Artifact) -> Result<()> {
        Self::delete_file(&artifact.file_name).await?;
        Ok(())
    }

    /// Store file to the filesystem storage.
    #[tracing::instrument]
    async fn store_file(
        local_filename: impl ToString + std::fmt::Debug,
    ) -> Result<String> {
        let root_folder = PathBuf::from(ROOT_FOLDER);
        let local_filename = local_filename.to_string();

        tokio::fs::copy(&local_filename, root_folder.join(&local_filename)).await?;
        Ok(root_folder.join(&local_filename).display().to_string())
    }

    #[tracing::instrument]
    async fn delete_file(filename: impl ToString + std::fmt::Debug) -> Result<()> {
        let root_folder = PathBuf::from(ROOT_FOLDER);
        let filename = filename.to_string();

        if let Err(e) = tokio::fs::remove_file(root_folder.join(&filename)).await {
            tracing::warn!(
                "Failed to delete file '{}' due to: {}",
                root_folder.join(&filename).display(),
                e
            );
        }

        Ok(())
    }

    pub fn get_download_url(filename: &str) -> String {
        let root_folder = PathBuf::from(ROOT_FOLDER);
        root_folder
            .join(&filename)
            .display()
            .to_string()
            .replace("\\", "/")
    }
}
