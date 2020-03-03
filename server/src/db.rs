use crate::{
    models::{Channel, Platform},
    Result,
};

// TODO: Needs to track releases too!
// TODO: Simply store it in a ron file lol.

pub fn init() -> Result<Database> {
    Ok(Database {
        db: sled::open(crate::config::DATABASE_FILE)?,
    })
}

#[derive(Clone)]
pub struct Database {
    db: sled::Db,
}

/// Path documentation of the database:
/// version.<latest>.<platform>.<channel> holds version hash
/// uri.<latest>.<platform>.<channel> holds the download url
///
/// Latest is hardcoded for now till support for multiple nightlies is there.

impl Database {
    pub fn get_latest_channel_version(&self, platform: Platform, channel: Channel) -> Result<Option<String>> {
        use std::str;
        Ok(self
            .db
            .get(&format!("version.latest.{}.{}", platform, channel))?
            .map(|x| str::from_utf8(&x).expect("FATAL: Corrupted database!").to_string()))
    }

    pub fn get_latest_uri(&self, platform: Platform, channel: Channel) -> Result<Option<String>> {
        use std::str;
        Ok(self
            .db
            .get(&format!("uri.latest.{}.{}", platform, channel))?
            .map(|x| str::from_utf8(&x).expect("FATAL: Corrupted database!").to_string()))
    }

    pub fn update_artifact(&mut self, platform: Platform, channel: Channel, hash: &str, download_uri: &str) -> Result<()> {
        self.db.insert(&format!("version.latest.{}.{}", platform, channel), hash)?;
        self.db.insert(&format!("uri.latest.{}.{}", platform, channel), download_uri)?;
        Ok(())
    }
}
