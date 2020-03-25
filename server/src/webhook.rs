use crate::{error::ServerError, models::Artifact, Result, CONFIG};
use tracing_futures::Instrument;

pub fn process(artifacts: Vec<Artifact>, mut db: crate::DbConnection) {
    async_std::task::spawn(
        async move {
            for artifact in artifacts {
                if let Err(e) = download(artifact, &mut db).await {
                    tracing::error!("Failed to download artifact: {}.", e);
                }
            }
            if let Err(e) = crate::prune::prune(&mut db).await {
                tracing::error!("Pruning failed: {}.", e);
            }
        }
        .instrument(tracing::info_span!("PipelineUpdate")),
    );
}

#[tracing::instrument(skip(db))]
async fn download(artifact: Artifact, db: &mut crate::DbConnection) -> Result<()> {
    use async_std::fs::File;

    tracing::info!("Downloading...");

    let req = isahc::get_async(&artifact.get_url()).await?;
    if req.status().as_u16() < 400 && req.status().as_u16() >= 200 {
        let mut f = File::create(&artifact.download_path).await?;
        async_std::io::copy(&mut req.into_body(), &mut f).await?;
    } else {
        return Err(ServerError::InvalidResponseCode(req.status(), artifact));
    }

    tracing::info!("Downloaded to {}", artifact.download_path.display());

    tracing::info!("Uploading artifact...");
    crate::S3Connection::new()?.upload(&artifact)?;

    // Delete obselete artifact
    let _ = std::fs::remove_file(&artifact.download_path);

    // Update database with new information
    db.update_artifact(
        artifact.date,
        &artifact.hash,
        artifact.platform,
        artifact.channel,
        &format!(
            "https://{}.{}/nightly/{}",
            CONFIG.bucket_name,
            CONFIG.bucket_endpoint,
            artifact.download_path.file_name().unwrap().to_string_lossy()
        ),
    )?;
    Ok(())
}
