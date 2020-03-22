use crate::{
    models::{Channel, Platform},
    Result,
};
use rocket_contrib::database;

#[database("sqlite")]
pub struct DbConnection(rusqlite::Connection);

impl DbConnection {
    pub fn get_latest_channel_version(&self, platform: Platform, channel: Channel) -> Result<Option<String>> {
        match self.0.query_row(
            &Self::table("SELECT hash FROM {table} WHERE platform = (?1) AND channel = (?2) ORDER BY date DESC LIMIT 1;"),
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
            &Self::table("SELECT download_uri FROM {table} WHERE platform = (?1) AND channel = (?2) ORDER BY date DESC LIMIT 1;"),
            &[&platform.to_string(), &channel.to_string()],
            |row| row.get(0),
        ) {
            Ok(uri) => Ok(Some(uri)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update_artifact(
        &mut self,
        date: chrono::NaiveDateTime,
        hash: &str,
        platform: Platform,
        channel: Channel,
        download_uri: &str,
    ) -> Result<()> {
        self.0.execute(
            &Self::table(
                "INSERT INTO {table} (date, hash, platform, channel, download_uri)
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
        Ok(())
    }

    pub fn table(query: &str) -> String {
        query.replace("{table}", crate::config::AIRSHIPPER_TABLE)
    }
}
