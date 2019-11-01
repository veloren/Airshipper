use rocket::http::Status;
use rocket::response::Redirect;
use rocket::*;

use crate::db::DbConnection;
use crate::error::ServerError;
use crate::models::{Channel, Platform};
use crate::Result;

// If no channel specified we default to nightly.
// NOTE: We want to change this behaviour once stable releases are more used than nightly
#[get("/version/<platform>")]
pub fn version(conn: DbConnection, platform: Option<Platform>) -> Result<String> {
    match platform {
        Some(platform) => match conn.get_latest_channel_version(platform, Channel::Nightly)? {
            Some(ver) => Ok(ver),
            None => Err(Status::NotFound.into()),
        },
        None => Err(ServerError::InvalidPlatform),
    }
}

#[get("/version/<platform>/<channel>")]
pub fn channel_version(
    conn: DbConnection,
    platform: Option<Platform>,
    channel: Option<Channel>,
) -> Result<String> {
    match platform {
        Some(platform) => match channel {
            Some(channel) => match conn.get_latest_channel_version(platform, channel)? {
                Some(ver) => Ok(ver),
                None => Err(Status::NotFound.into()),
            },
            None => Err(ServerError::InvalidChannel),
        },
        None => Err(ServerError::InvalidPlatform),
    }
}

// If no channel specified we default to nightly.
// NOTE: We want to change this behaviour once stable releases are more used than nightly
#[get("/latest/<platform>")]
pub fn download(conn: DbConnection, platform: Option<Platform>) -> Result<Redirect> {
    match platform {
        Some(platform) => match conn.get_latest_path(platform, Channel::Nightly)? {
            Some(path) => Ok(Redirect::to(path)),
            None => Err(Status::NotFound.into()),
        },
        None => Err(ServerError::InvalidPlatform),
    }
}

#[get("/latest/<platform>/<channel>")]
pub fn channel_download(
    conn: DbConnection,
    platform: Option<Platform>,
    channel: Option<Channel>,
) -> Result<Redirect> {
    match platform {
        Some(platform) => match channel {
            Some(channel) => match conn.get_latest_path(platform, channel)? {
                Some(path) => Ok(Redirect::to(path)),
                None => Err(Status::NotFound.into()),
            },
            None => Err(ServerError::InvalidChannel),
        },
        None => Err(ServerError::InvalidPlatform),
    }
}
