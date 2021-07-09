use crate::{FsStorage, Result};

/// Prunes local db and S3 storage from old nightlies.
#[tracing::instrument(skip(db))]
pub async fn prune(db: &mut crate::DbConnection) -> Result<()> {
    if db.has_pruneable_artifacts().await? {
        let artifacts = db.prune_artifacts().await?;

        for artifact in artifacts {
            tracing::info!("Deleting prunable artifact: {:?}", artifact);
            FsStorage::delete(&artifact).await?;
        }
    }
    Ok(())
}
