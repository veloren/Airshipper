use crate::Result;

/// Prunes local db and S3 storage from old nightlies.
#[tracing::instrument(skip(db))]
pub async fn prune(db: &mut crate::DbConnection) -> Result<()> {
    if db.has_pruneable_artifacts()? {
        let s3con = crate::S3Connection::new()?;
        let artifacts = db.prune_artifacts()?;

        for artifact in artifacts {
            tracing::info!("Deleting prunable artifact: {:?}", artifact);
            s3con.delete(&artifact).await?;
        }
    }
    Ok(())
}
