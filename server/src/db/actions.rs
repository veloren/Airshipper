use sqlx::{any::AnyArguments, Any, Executor, QueryBuilder, Row, Transaction};

use crate::{db::Db, models::Artifact, FsStorage, Result};

#[tracing::instrument(skip(db))]
pub async fn artifacts_exist(db: &Db, cmp: &[Artifact]) -> Result<bool> {
    let uris: Vec<String> = cmp.iter().map(|x| &x.download_uri).cloned().collect();

    let args = AnyArguments::default();
    let mut query_builder = QueryBuilder::with_arguments(
        r"SELECT COUNT(id) as cnt
        FROM artifacts
        WHERE (download_uri) IN ",
        args,
    );
    query_builder.push_tuples(uris, |mut b, uri| {
        b.push_bind(uri);
    });

    let row = query_builder.build().fetch_one(&db.pool).await?;
    let count: i64 = row.try_get("cnt")?;

    Ok(count == cmp.len() as i64)
}

#[tracing::instrument(skip(db))]
pub async fn insert_artifact(db: &Db, artifact: &Artifact) -> Result<i64> {
    // TODO: check if the following TODO still is wanted behavior
    // TODO: Check whether UNIQUE constraint gets violated and throw a warning but
    // proceed!
    let query = sqlx::query(
        r"INSERT INTO artifacts (build_id, date, hash, author, merged_by, os, arch, channel, file_name, download_uri) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING ID",
    ).bind(artifact.build_id)
    .bind(artifact.date.to_string())
    .bind(&artifact.hash)
    .bind(&artifact.author)
    .bind(&artifact.merged_by)
    .bind(&artifact.os)
    .bind(&artifact.arch)
    .bind(&artifact.channel)
    .bind(&artifact.file_name)
    .bind(&artifact.download_uri);

    let row = db.pool.fetch_one(query).await?;
    let id: i64 = row.try_get("id")?;
    Ok(id)
}

pub struct VersionUri {
    pub version: String,
    pub uri: String,
}

#[tracing::instrument(skip(db))]
pub async fn get_latest_version_uri<
    T: ToString + std::fmt::Debug,
    U: ToString + std::fmt::Debug,
    Y: ToString + std::fmt::Debug,
>(
    db: &Db,
    searched_os: T,
    searched_arch: U,
    searched_channel: Y,
) -> Result<Option<VersionUri>> {
    let searched_os = searched_os.to_string().to_lowercase();
    let searched_arch = searched_arch.to_string().to_lowercase();
    let searched_channel = searched_channel.to_string().to_lowercase();

    let query = sqlx::query(
        r"SELECT hash, download_uri FROM artifacts WHERE os = ? AND arch = ? AND channel = ? ORDER BY date DESC;",
    )
    .bind(&searched_os)
    .bind(&searched_arch)
    .bind(&searched_channel);

    let row = db.pool.fetch_optional(query).await?;
    match row {
        Some(row) => Ok(Some(VersionUri {
            version: row.try_get("hash")?,
            uri: row.try_get("download_uri")?,
        })),
        None => Ok(None),
    }
}

/// Prunes local db and S3 storage from old nightlies.
#[tracing::instrument(skip(db))]
pub async fn prune(db: &Db) -> Result<()> {
    let mut con = db.pool.begin().await?;

    if has_pruneable_artifacts(&mut con).await? {
        let files = prune_artifacts(&mut con).await?;

        for file in files {
            tracing::info!("Deleting prunable artifact: {:?}", file);
            FsStorage::delete_file(&file).await;
        }
    }
    con.commit().await?;

    Ok(())
}

async fn has_pruneable_artifacts(con: &mut Transaction<'static, Any>) -> Result<bool> {
    let query = sqlx::query("SELECT COUNT(id) as cnt FROM artifacts");
    let row = con.fetch_one(query).await?;

    let count: i64 = row.try_get("cnt")?;
    Ok(count > 20)
}

/// Prunes all artifacts but one per os/arch/channel combination
async fn prune_artifacts(con: &mut Transaction<'static, Any>) -> Result<Vec<String>> {
    let query = sqlx::query(
        "DELETE FROM artifacts
    WHERE id NOT IN
    (
        SELECT MIN(id)
        FROM artifacts
        GROUP BY channel, os, arch
    ) RETURNING file_name",
    );
    let rows = con.fetch_all(query).await?;

    let mut files = Vec::new();
    for row in rows {
        let file_name = row.try_get("file_name")?;
        files.push(file_name);
    }
    Ok(files)
}
