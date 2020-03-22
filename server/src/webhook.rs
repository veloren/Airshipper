use crate::{
    models::{Artifact, PipelineUpdate},
    Result, CONFIG,
};
use rocket::http::ContentType;
use std::thread::JoinHandle;

/// NOTE: This just spawns another thread! This might break if multiple
/// pipeline updates get received in short time (shouldn't happen at all due to compile time)
/// TODO: Make it an async task!
pub fn process(update: PipelineUpdate, mut db: crate::DbConnection) -> JoinHandle<()> {
    std::thread::spawn(move || {
        tracing::info!(
            "[1] Received new update: {} - {}",
            short_desc(&update),
            update.commit.timestamp.format("%Y-%m-%d | %H:%M:%S")
        );
        match extract_all(update) {
            Ok(artifacts) => {
                if artifacts.is_empty() {
                    tracing::info!("[2] No artifacts found.");
                    return;
                }

                for artifact in artifacts {
                    tracing::info!(
                        "[2] Downloading {}-{} merged by {}",
                        artifact.channel,
                        artifact.platform,
                        artifact.merged_by
                    );
                    if let Err(e) = &artifact.download() {
                        tracing::error!("Encountered error while downloading artifact: {:?}", e);
                        return;
                    }
                    tracing::info!("[3] Downloaded to {}", artifact.download_path.display());
                    // NOTE: Only 3 artifacts should exist. So they will need to be overwritten and db needs to keep
                    // track of them too.
                    tracing::info!("[4] Uploading artifact...");
                    let credentials = s3::credentials::Credentials::new(
                        Some(CONFIG.bucket_access_key.clone()),
                        Some(CONFIG.bucket_secret_key.clone()),
                        None,
                        None,
                    );
                    let mut bucket =
                        match s3::bucket::Bucket::new(&CONFIG.bucket_name, CONFIG.bucket_region.clone(), credentials) {
                            Ok(x) => x,
                            Err(e) => {
                                tracing::error!("Encountered error while uploading artifact: {:?}", e);
                                return;
                            },
                        };
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
                    tracing::info!("[5] Bucket responded with {}", code);

                    // Update database with new information
                    if let Err(e) = db.update_artifact(
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
                    ) {
                        tracing::error!("Encountered error when inserting an artifact: {:?}", e);
                        return;
                    }
                    // Delete obselete artifact
                    std::fs::remove_file(artifact.download_path).expect("Failed to clean up artifact!");
                }
            },
            Err(e) => tracing::info!("[2] Failed to process PipelineUpdate! {:?}", e),
        }
    })
}

/// Returns all artifacts which needs to be downloaded
fn extract_all(pipe: PipelineUpdate) -> Result<Vec<Artifact>> {
    let mut artifacts = Vec::new();

    for build in &pipe.builds {
        // Skip non-artifact builds.
        if build.stage != crate::CONFIG.artifact_stage {
            continue;
        }

        if let Some(artifact) = Artifact::try_from(&pipe, build)? {
            artifacts.push(artifact);
        }
    }

    Ok(artifacts)
}

/// Returns a short preview of the commit message
fn short_desc(update: &PipelineUpdate) -> String {
    update.commit.message[..update.commit.message.find("\n").unwrap_or(update.commit.message.len())]
        .chars()
        .take(40)
        .collect()
}
