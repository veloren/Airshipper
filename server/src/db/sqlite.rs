use super::schema;
use crate::{models::Artifact, Result};
use diesel::prelude::*;
use rocket_contrib::database;

#[database("sqlite")]
pub struct DbConnection(diesel::SqliteConnection);

#[derive(Debug, Queryable)]
pub struct DbArtifact {
    pub id: i32,
    pub build_id: i32,
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
    pub fn get_latest_version<T: ToString>(&self, searched_platform: T, searched_channel: T) -> Result<Option<String>> {
        use schema::artifacts::dsl::*;
        Ok(artifacts
            .select(hash)
            .order(date.desc())
            .filter(platform.eq(searched_platform.to_string().to_lowercase()))
            .filter(channel.eq(searched_channel.to_string().to_lowercase()))
            .first(&self.0)
            .optional()?)
    }

    pub fn get_latest_uri<T: ToString>(&self, searched_platform: T, searched_channel: T) -> Result<Option<String>> {
        use schema::artifacts::dsl::*;
        Ok(artifacts
            .select(download_uri)
            .order(date.desc())
            .filter(platform.eq(searched_platform.to_string().to_lowercase()))
            .filter(channel.eq(searched_channel.to_string().to_lowercase()))
            .first(&self.0)
            .optional()?)
    }

    pub fn insert_artifact(&mut self, new_artifact: &Artifact) -> Result<()> {
        use schema::artifacts;
        // TODO: Check whether UNIQUE constraint gets violated and throw a warning but proceed!
        diesel::insert_or_ignore_into(artifacts::table)
            .values(new_artifact)
            .execute(&self.0)?;
        Ok(())
    }

    pub fn has_pruneable_artifacts(&self) -> Result<bool> {
        use schema::artifacts::dsl::*;
        let count: Option<i64> = artifacts.count().get_result(&self.0).optional()?;
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

    pub fn prune_artifacts(&self) -> Result<Vec<Artifact>> {
        use schema::artifacts::dsl::*;
        let artis = artifacts
            .order(date.desc())
            .limit(1000)
            .offset(6)
            .load::<DbArtifact>(&self.0)?;

        let ids: Vec<i32> = artis.iter().map(|x| x.id).collect();
        diesel::delete(artifacts.filter(id.eq_any(ids))).execute(&self.0)?;
        Ok(artis.iter().map(|x| x.into()).collect())
    }

    pub fn does_not_exist(&self, cmp: &[Artifact]) -> Result<bool> {
        use schema::artifacts::dsl::*;
        let uris: Vec<&String> = cmp.iter().map(|x| &x.download_uri).collect();
        let count: Option<i64> = artifacts
            .filter(download_uri.eq_any(uris))
            .count()
            .get_result(&self.0)
            .optional()?;
        match count {
            Some(0) => Ok(true),
            Some(_) => Ok(false),
            None => Ok(true),
        }
    }
}
