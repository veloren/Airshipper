use crate::{error::ServerError, models::Artifact, Result, CONFIG};
use rocket::http::ContentType;
use tracing_futures::Instrument;

pub fn process(artifacts: Vec<Artifact>, mut db: crate::DbConnection) {
    async_std::task::spawn(
        async move {
            for artifact in artifacts {
                if let Err(e) = download(artifact, &mut db).await {
                    tracing::error!("Failed to download artifact: {}.", e);
                }
            }
        }
        .instrument(tracing::info_span!("PipelineUpdate")),
    );
}

#[tracing::instrument(skip(db))]
async fn download(artifact: Artifact, db: &mut crate::DbConnection) -> Result<()> {
    use async_std::fs::File;
    use s3::{bucket::Bucket, credentials::Credentials};

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
    let credentials = Credentials::new(
        Some(CONFIG.bucket_access_key.clone()),
        Some(CONFIG.bucket_secret_key.clone()),
        None,
        None,
    );
    let mut bucket = Bucket::new(&CONFIG.bucket_name, CONFIG.bucket_region.clone(), credentials)?;
    bucket.add_header("x-amz-acl", "public-read");

    let (_, code) = bucket
        .put_object(
            &format!(
                "/nightly/{}",
                &artifact.download_path.file_name().unwrap().to_string_lossy()
            ), /* Unwrap safe. We always
                * have a file extension! */
            &std::fs::read(&artifact.download_path).expect("Failed to read file for upload!"),
            &ContentType::from_extension(
                &artifact
                    .download_path
                    .extension()
                    .unwrap_or(std::ffi::OsStr::new("zip"))
                    .to_string_lossy(),
            )
            .unwrap_or(ContentType::ZIP)
            .to_string(),
        )
        .expect("Failed to upload!");
    tracing::info!("Bucket responded with {}", code); // TODO: Check if that code is success!

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
    // Delete obselete artifact if it exists
    let _ = std::fs::remove_file(artifact.download_path);
    Ok(())
}
