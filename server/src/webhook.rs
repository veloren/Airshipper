use crate::{models::Artifact, Result};

pub fn process(artifacts: Vec<Artifact>, mut db: crate::DbConnection) {
    tokio::task::spawn(async move {
        for artifact in artifacts {
            if let Err(e) = download(artifact, &mut db).await {
                tracing::error!("Failed to download artifact: {}.", e);
            }
        }
        if let Err(e) = crate::prune::prune(&mut db).await {
            tracing::error!("Pruning failed: {}.", e);
        }
    });
}

#[tracing::instrument(skip(db))]
async fn download(artifact: Artifact, db: &mut crate::DbConnection) -> Result<()> {
    use tokio::{fs::File, prelude::*};

    tracing::info!("Downloading...");

    let mut resp = reqwest::get(&artifact.get_url()).await?;
    let mut file = File::create(&artifact.file_name).await?;
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
    }

    tracing::info!("Uploading...");
    crate::S3Connection::new()?.upload(&artifact).await?;

    // Delete obselete artifact
    let _ = std::fs::remove_file(&artifact.file_name);

    // Update database with new information
    tracing::info!("Update database...");
    db.insert_artifact(artifact)?;
    Ok(())
}
