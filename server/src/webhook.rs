use crate::{error::ServerError, models::Artifact, Result};
use reqwest::StatusCode;

pub fn process(artifacts: Vec<Artifact>, mut db: crate::DbConnection) {
    tokio::task::spawn(async move {
        for artifact in artifacts {
            if let Err(e) = transfer(artifact, &mut db).await {
                tracing::error!("Failed to transfer artifact: {}.", e);
            }
        }
        if let Err(e) = crate::prune::prune(&mut db).await {
            tracing::error!("Pruning failed: {}.", e);
        }
    });
}

#[tracing::instrument(skip(db))]
async fn transfer(artifact: Artifact, db: &mut crate::DbConnection) -> Result<()> {
    use tokio::{fs::File, prelude::*};

    tracing::info!("Downloading...");

    let mut resp = reqwest::get(&artifact.get_url()).await?;
    let mut file = File::create(&artifact.file_name).await?;
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
    }

    tracing::info!("Uploading...");
    let code = crate::S3Connection::new()?.upload(&artifact).await?;

    // Delete obselete artifact
    let _ = std::fs::remove_file(&artifact.file_name);

    if is_success(code) {
        // Update database with new information
        tracing::info!("Update database...");
        db.insert_artifact(artifact)?;
        Ok(())
    } else {
        Err(ServerError::InvalidResponseCode(
            StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            artifact,
        ))
    }
}

fn is_success(code: u16) -> bool {
    if code < 399 && code > 199 { true } else { false }
}
