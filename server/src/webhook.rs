use std::thread::JoinHandle;

use crate::{
    models::{Artifact, PipelineUpdate},
    Result, CONFIG,
};

/// NOTE: This just spawns another thread! This might break if multiple
/// pipeline updates get received in short time (shouldn't happen at all due to compile time)
/// TODO: Replace with static background thread which communicates via channels.
pub fn process(update: PipelineUpdate, mut db: crate::DbConnection) -> JoinHandle<()> {
    std::thread::spawn(move || {
        log::info!(
            "[1] Received new update: {} - {}",
            short_desc(&update),
            update.commit.timestamp.format("%Y-%m-%d | %H:%M:%S")
        );
        match extract_all(update) {
            Ok(artifacts) => {
                if artifacts.is_empty() {
                    log::info!("[2] No artifacts found.");
                    return;
                }

                for artifact in artifacts {
                    log::info!(
                        "[2] Downloading {}-{} merged by {}",
                        artifact.channel,
                        artifact.platform,
                        artifact.merged_by
                    );
                    if let Err(e) = &artifact.download() {
                        log::error!("Encountered error while downloading artifact: {:?}", e);
                        return;
                    }
                    log::info!("[3] Downloaded to {}", artifact.download_path.display());
                    // NOTE: Only 3 artifacts should exist. So they will need to be overwritten and db needs to keep track of them too.
                    log::info!("[4] Uploading artifact...");
                    let credentials =
                        s3::credentials::Credentials::new(Some(CONFIG.bucket_access_key.clone()), Some(CONFIG.bucket_secret_key.clone()), None, None);
                    let mut bucket = match s3::bucket::Bucket::new(&CONFIG.bucket_name, CONFIG.bucket_region.clone(), credentials) {
                        Ok(x) => x,
                        Err(e) => {
                            log::error!("Encountered error while uploading artifact: {:?}", e);
                            return;
                        },
                    };
                    bucket.add_header("x-amz-acl", "public-read");

                    let (_, code) = bucket
                        .put_object(
                            &format!("/{:?}", &artifact.download_path.file_name().unwrap()),
                            "ABC".as_bytes(), // TODO: Actual content
                            "text/plain",     // TODO: Actual content type
                        )
                        .unwrap();
                    log::info!("[4] Bucket responded with {}", code);

                    // Update database with new information
                    if let Err(e) = db.update_artifact(
                        artifact.date,
                        &artifact.hash,
                        artifact.platform,
                        artifact.channel,
                        &format!(
                            "https://{}.{}.{}/{:?}",
                            CONFIG.bucket_name,
                            CONFIG.bucket_region,
                            CONFIG.bucket_endpoint,
                            artifact.download_path.file_name().unwrap()
                        ),
                    ) {
                        log::error!("Encountered error when inserting an artifact: {:?}", e);
                        return;
                    }
                }
            },
            Err(e) => log::info!("[2] Failed to process PipelineUpdate! {:?}", e),
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
