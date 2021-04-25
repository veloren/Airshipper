use crate::{models::Artifact, FsStorage, Result};
use rocket::tokio;

pub fn process(artifacts: Vec<Artifact>, mut db: crate::DbConnection) {
    tokio::spawn(async move {
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
    use tokio::{fs::File, io::AsyncWriteExt};

    tracing::info!("Downloading...");

    let mut resp = reqwest::get(&artifact.get_url()).await?;
    let mut file = File::create(&artifact.file_name).await?;
    let mut content = vec![];
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
        content.write_all(&chunk).await?;
    }
    file.sync_data().await?;

    let hash = format!("{:x}", md5::compute(content));
    let remote_hash = get_remote_hash(&resp);

    if hash != remote_hash {
        tracing::error!(
            "Downloaded file has '{}' MD5 hash while remote hash is '{}'. Exiting...",
            hash,
            remote_hash
        );
        // Clean up
        tokio::fs::remove_file(&artifact.file_name).await?;
    } else {
        tracing::debug!("Computed hash: {}, remote_hash: {}", hash, remote_hash);
        tracing::info!("Storing...");

        FsStorage::store(&artifact).await?;

        // Update database with new information
        tracing::info!("hash valid. Update database...");
        db.insert_artifact(&artifact).await?;

        // Delete obselete artifact
        tokio::fs::remove_file(&artifact.file_name).await?;
    }
    Ok(())
}

fn get_remote_hash(resp: &reqwest::Response) -> String {
    resp.headers()
        .get(reqwest::header::ETAG)
        .map(|x| x.to_str().expect("always valid ascii?"))
        .unwrap_or("REMOTE_ETAG_MISSING")
        .replace("\"", "")
}
