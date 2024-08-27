use sqlx::{Any, Executor, QueryBuilder, Row, Transaction};

use crate::{
    db::Db,
    error::{ProcessError, ServerError},
    models::Artifact,
    FsStorage,
};

#[tracing::instrument(skip(db))]
pub async fn any_artifacts_exist(db: &Db, cmp: &[Artifact]) -> Result<bool, ServerError> {
    let uris = cmp.iter().map(|x| &x.download_uri);

    let mut query_builder = QueryBuilder::new(
        r"SELECT COUNT(id) as cnt
        FROM artifacts
        WHERE (download_uri) IN ",
    );
    query_builder.push_tuples(uris, |mut b, uri| {
        b.push_bind(uri);
    });

    let count: i64 = query_builder
        .build_query_scalar()
        .fetch_one(&db.pool)
        .await?;

    Ok(count > 0)
}

#[tracing::instrument(skip(db))]
pub async fn insert_artifact(db: &Db, artifact: &Artifact) -> Result<i64, ProcessError> {
    let query = sqlx::query_scalar(
        r"INSERT INTO artifacts (build_id, date, hash, author, merged_by, os, arch, channel, file_name, download_uri) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
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

    let id: i64 = query.fetch_one(&db.pool).await?;
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
) -> Result<Option<VersionUri>, ServerError> {
    let searched_os = searched_os.to_string().to_lowercase();
    let searched_arch = searched_arch.to_string().to_lowercase();
    let searched_channel = searched_channel.to_string().to_lowercase();

    let query = sqlx::query(
        r"SELECT hash, download_uri
          FROM artifacts
          WHERE os = ? AND arch = ? AND channel = ?
          ORDER BY date DESC
          LIMIT 1;",
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
pub async fn prune(db: &Db) -> Result<(), ServerError> {
    let mut con = db.pool.begin().await?;

    let files = prune_artifacts(&mut con).await?;

    for file in files {
        tracing::info!("Deleting prunable artifact: {:?}", file);
        FsStorage::delete_file(&file).await;
    }

    con.commit().await?;

    Ok(())
}

/// Prunes all artifacts but one per os/arch/channel combination
#[allow(unused_variables, unreachable_code)]
async fn prune_artifacts(
    con: &mut Transaction<'static, Any>,
) -> Result<Vec<String>, ServerError> {
    // TODO: fix the sql query
    return Ok(vec![]);

    // Currently date is a STRING and the order DESC might cause weird effects IF we
    // would store different timezones.
    let query = sqlx::query(
        "DELETE FROM artifacts
    WHERE id NOT IN
    (
        SELECT id
        FROM artifacts
        GROUP BY channel, os, arch
        ORDER BY date DESC
    ) RETURNING file_name",
    );
    let rows = con.fetch_all(query).await?;

    if !rows.is_empty() {
        tracing::info!("pruned artifacts from db");
    }

    let mut files = Vec::new();
    for row in rows {
        let file_name = row.try_get("file_name")?;
        files.push(file_name);
    }
    Ok(files)
}
