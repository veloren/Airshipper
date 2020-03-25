use crate::{
    models::{Channel, Platform},
    Result,
};
use rocket_contrib::database;
use rusqlite::NO_PARAMS;

#[database("sqlite")]
pub struct DbConnection(rusqlite::Connection);

impl DbConnection {
    pub fn create_table(&self) -> Result<()> {
        self.0.execute_batch(&DbConnection::table(
            "CREATE TABLE IF NOT EXISTS {table} (
                        date timestamp without time zone NOT NULL,
                        hash varchar NOT NULL,
                        platform varchar NOT NULL,
                        channel varchar NOT NULL,
                        download_uri varchar NOT NULL PRIMARY KEY
                    );",
        ))?;
        Ok(())
    }

    pub fn get_latest_channel_version(&self, platform: Platform, channel: Channel) -> Result<Option<String>> {
        match self.0.query_row(
            &Self::table(
                "SELECT hash FROM {table} WHERE platform = (?1) AND channel = (?2) ORDER BY date DESC LIMIT 1;",
            ),
            &[&platform.to_string(), &channel.to_string()],
            |row| row.get(0),
        ) {
            Ok(version) => Ok(Some(version)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_latest_uri(&self, platform: Platform, channel: Channel) -> Result<Option<String>> {
        match self.0.query_row(
            &Self::table(
                "SELECT download_uri FROM {table} WHERE platform = (?1) AND channel = (?2) ORDER BY date DESC LIMIT 1;",
            ),
            &[&platform.to_string(), &channel.to_string()],
            |row| row.get(0),
        ) {
            Ok(uri) => Ok(Some(uri)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn update_artifact(
        &mut self,
        date: chrono::NaiveDateTime,
        hash: &str,
        platform: Platform,
        channel: Channel,
        download_uri: &str,
    ) -> Result<()> {
        tracing::debug!("Inserting into db...");
        self.0.execute(
            &Self::table(
                "INSERT OR IGNORE INTO {table} (date, hash, platform, channel, download_uri)
                        VALUES (?1, ?2, ?3, ?4, ?5);",
            ),
            vec![
                date.to_string(),
                hash.to_string(),
                platform.to_string(),
                channel.to_string(),
                download_uri.to_string(),
            ],
        )?;
        tracing::debug!("Done.");
        Ok(())
    }

    pub fn has_pruneable_artifacts(&self) -> Result<bool> {
        match self
            .0
            .query_row(&Self::table("SELECT COUNT(*) FROM {table}"), NO_PARAMS, |row| {
                row.get::<_, i64>(0)
            }) {
            Ok(candidates) => Ok(if candidates > 10 { true } else { false }),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    // Returns a list of the urls of the pruned artifacts.
    /*#[tracing::instrument(skip(self))]
    pub fn prune_artifacts(&mut self) -> Result<Vec<Artifact>> {
        let tx = self.0.transaction()?;
        let mut pruneable_artifacts = Vec::new();
        {
            let mut query = tx.prepare(&Self::table(
                "Select date,platform,channel,download_uri FROM {table} ORDER BY date DESC LIMIT 100 OFFSET 6;",
            ))?;
            let rows = query.query_map(NO_PARAMS, |row| row.get(0))?;

            Artifact::get_download_path(date, platform, channel, file_ending);

            for pruneable in rows {
                pruneable_artifacts.push(pruneable?);
            }

            let mut delete = tx.prepare(&Self::table("DELETE FROM {table} WHERE download_uri = (?)"))?;
            delete.execute(&pruneable_artifacts)?;
        }
        tx.commit()?;

        Ok(pruneable_artifacts)
    }*/

    pub fn table(query: &str) -> String {
        query.replace("{table}", crate::config::AIRSHIPPER_TABLE)
    }
}
