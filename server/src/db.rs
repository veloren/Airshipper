use crate::models::{Artifact, Channel, Platform};
use crate::Result;
use rocket_contrib::database;

#[database("postgres")]
pub struct DbConnection(postgres::Connection);

impl DbConnection {
    /// Returns Path to the latest nightly on disk.
    pub fn get_latest_path(&self, platform: Platform, channel: Channel) -> Result<Option<String>> {
        // TODO: Why isn't there a query_one :(
        for row in self
            .0
            .query(
                &Self::table(
                    "SELECT download_path FROM {} WHERE platform = $1 AND channel = $2 ORDER BY date DESC LIMIT 1;",
                ),
                &[&platform.to_string(), &channel.to_string()],
            )?
            .iter()
        {
            return Ok(Some(format!(
                "/{}",
                row.get::<&str, String>("download_path")
            )));
        }
        Ok(None)
    }
    
    /// Returns Path to the latest version on disk.
    pub fn get_latest_channel_version(
        &self,
        platform: Platform,
        channel: Channel,
    ) -> Result<Option<String>> {
        // TODO: Why isn't there a query_one :(
        for row in self
            .0
            .query(
                &Self::table("SELECT hash FROM {} WHERE platform = $1 AND channel = $2 ORDER BY date DESC LIMIT 1;"),
                &[&platform.to_string(), &channel.to_string()],
            )?
            .iter()
        {
            return Ok(Some(row.get("hash")));
        }
        Ok(None)
    }
    
    /// Insert a new artifact into the database with it's metadata
    pub fn insert_artifact(&self, artifact: Artifact) -> Result<()> {
        self.0.execute(
            &Self::table(
                "INSERT INTO {} (date, hash, author, merged_by, platform, channel, download_path) 
                        VALUES ($1, $2, $3, $4, $5, $6, $7);",
            ),
            &[
                &artifact.date,
                &artifact.hash,
                &artifact.author,
                &artifact.merged_by,
                &artifact.platform.to_string(),
                &artifact.channel.to_string(),
                &artifact.download_path.display().to_string(),
            ],
        )?;
        Ok(())
    }

    // Required because postgres does not like parameters to be a table name
    pub fn table(x: &str) -> String {
        x.replace("{}", &crate::CONFIG.database_table)
    }
}
