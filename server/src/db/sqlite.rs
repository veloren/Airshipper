use crate::{models::Artifact, Result};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct DbConnection(SqlitePool);

#[derive(Debug, sqlx::FromRow)]
pub struct DbArtifact {
    pub id: i64,
    pub build_id: i64,
    pub date: chrono::NaiveDateTime,
    pub hash: String,
    pub author: String,
    pub merged_by: String,

    pub platform: String,
    pub channel: String,
    pub file_name: String,
    pub download_uri: String,
}

impl DbConnection {
    pub fn new(pool: SqlitePool) -> Self {
        Self(pool)
    }

    pub async fn get_latest_version<T: ToString, Y: ToString>(
        &self,
        searched_platform: T,
        searched_channel: Y,
    ) -> Result<Option<String>> {
        let searched_platform = searched_platform.to_string().to_lowercase();
        let searched_channel = searched_channel.to_string().to_lowercase();

        Ok(sqlx::query!(
            "SELECT hash FROM artifacts WHERE platform = $1 AND channel = $2 ORDER BY \
             date DESC;",
            searched_platform,
            searched_channel
        )
        .map(|s| s.hash)
        .fetch_one(&self.0)
        .await
        .ok())
    }

    pub async fn get_latest_uri<T: ToString, Y: ToString>(
        &self,
        searched_platform: T,
        searched_channel: Y,
    ) -> Result<Option<String>> {
        let searched_platform = searched_platform.to_string().to_lowercase();
        let searched_channel = searched_channel.to_string().to_lowercase();

        let uri: Option<String> = sqlx::query!(
            "SELECT download_uri FROM artifacts WHERE platform = $1 AND channel = $2 \
             ORDER BY date DESC;",
            searched_platform,
            searched_channel
        )
        .map(|s| s.download_uri)
        .fetch_one(&self.0)
        .await
        .ok();

        Ok(uri)
    }

    pub async fn insert_artifact(&mut self, new_artifact: &Artifact) -> Result<()> {
        // TODO: Check whether UNIQUE constraint gets violated and throw a warning but
        // proceed!
        let _ = sqlx::query!(
            "INSERT INTO artifacts (build_id, date, hash, author, merged_by, platform, \
             channel, file_name, download_uri) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, \
             $9);",
            new_artifact.build_id,
            new_artifact.date,
            new_artifact.hash,
            new_artifact.author,
            new_artifact.merged_by,
            new_artifact.platform,
            new_artifact.channel,
            new_artifact.file_name,
            new_artifact.download_uri,
        )
        .execute(&self.0)
        .await;
        Ok(())
    }

    pub async fn has_pruneable_artifacts(&self) -> Result<bool> {
        let count = sqlx::query!("SELECT COUNT(*) as count FROM artifacts;")
            .fetch_optional(&self.0)
            .await?
            .map(|s| s.count as i64);

        match count {
            Some(candidates) => {
                if candidates > 10 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            None => Ok(false),
        }
    }

    /// Prunes all artifacts but two per os
    pub async fn prune_artifacts(&self) -> Result<Vec<Artifact>> {
        // TODO: Query all platforms (SELECT DISTINCT) to not hardcode amount of OSes.
        let win_artis = sqlx::query_as!(
            DbArtifact,
            "SELECT * FROM artifacts WHERE platform = 'windows' ORDER BY date DESC \
             LIMIT 1,1000;"
        )
        .fetch_all(&self.0)
        .await?;
        let lin_artis = sqlx::query_as!(
            DbArtifact,
            "SELECT * FROM artifacts WHERE platform = 'linux' ORDER BY date DESC LIMIT \
             1,1000;"
        )
        .fetch_all(&self.0)
        .await?;
        let mac_artis = sqlx::query_as!(
            DbArtifact,
            "SELECT * FROM artifacts WHERE platform = 'macos' ORDER BY date DESC LIMIT \
             1,1000;"
        )
        .fetch_all(&self.0)
        .await?;

        let mut artis = vec![];
        artis.extend(win_artis);
        artis.extend(lin_artis);
        artis.extend(mac_artis);

        let ids = artis
            .iter()
            .map(|x| x.id.to_string())
            .collect::<Vec<String>>()
            .join(",");

        sqlx::query!("DELETE FROM artifacts WHERE id IN ($1)", ids)
            .execute(&self.0)
            .await?;

        Ok(artis.iter().map(|x| x.into()).collect())
    }

    pub async fn does_not_exist(&self, cmp: &[Artifact]) -> Result<bool> {
        let uris = cmp
            .iter()
            .map(|x| x.download_uri.clone())
            .collect::<Vec<String>>()
            .join(",");

        let count: Option<i64> = sqlx::query!(
            "SELECT COUNT(*) as count FROM artifacts WHERE download_uri IN ($1);",
            uris,
        )
        .fetch_optional(&self.0)
        .await?
        .map(|s| s.count as i64);

        match count {
            Some(0) => Ok(true),
            Some(_) => Ok(false),
            None => Ok(true),
        }
    }
}
