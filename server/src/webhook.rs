use std::thread::JoinHandle;

use crate::db::DbConnection;
use crate::models::{Artifact, PipelineUpdate};
use crate::Result;

/// NOTE: This just spawns another thread! This might break if multiple
/// pipeline updates get received in short time (shouldn't happen at all due to compile time)
/// TODO: Replace with static background thread which communicates via channels.
pub fn process(update: PipelineUpdate, conn: DbConnection) -> JoinHandle<()> {
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
                    if let Err(e) = artifact.download() {
                        log::error!("Encountered error while downloading artifact: {:?}", e);
                        return;
                    }
                    log::info!("[3] Downloaded to {}", artifact.download_path.display());

                    // Hopefully update database with new information
                    if let Err(e) = conn.insert_artifact(artifact) {
                        log::error!("Encountered error when inserting an artifact: {:?}", e);
                        return;
                    }
                }
            }
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
    update.commit.message[..update
        .commit
        .message
        .find("\n")
        .unwrap_or(update.commit.message.len())]
        .chars()
        .take(40)
        .collect()
}
