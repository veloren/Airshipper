use rocket::{http::Status, response::Redirect, *};

use crate::{
    error::ServerError,
    models::{Channel, Platform},
    Result,
};

// If no channel specified we default to nightly.
// NOTE: We want to change this behaviour once stable releases are more used than nightly
#[get("/version/<platform>")]
pub fn version(db: crate::DbConnection, platform: Option<Platform>) -> Result<String> {
    match platform {
        Some(platform) => match db.get_latest_channel_version(platform, Channel::Nightly)? {
            Some(ver) => Ok(ver),
            None => Err(Status::NotFound.into()),
        },
        None => Err(ServerError::InvalidPlatform),
    }
}

#[get("/version/<platform>/<channel>")]
pub fn channel_version(db: crate::DbConnection, platform: Option<Platform>, channel: Option<Channel>) -> Result<String> {
    match platform {
        Some(platform) => match channel {
            Some(channel) => match db.get_latest_channel_version(platform, channel)? {
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
pub fn download(db: crate::DbConnection, platform: Option<Platform>) -> Result<Redirect> {
    match platform {
        Some(platform) => match db.get_latest_uri(platform, Channel::Nightly)? {
            Some(uri) => Ok(Redirect::to(uri)),
            None => Err(Status::NotFound.into()),
        },
        None => Err(ServerError::InvalidPlatform),
    }
}

#[get("/latest/<platform>/<channel>")]
pub fn channel_download(db: crate::DbConnection, platform: Option<Platform>, channel: Option<Channel>) -> Result<Redirect> {
    match platform {
        Some(platform) => match channel {
            Some(channel) => match db.get_latest_uri(platform, channel)? {
                Some(uri) => Ok(Redirect::to(uri)),
                None => Err(Status::NotFound.into()),
            },
            None => Err(ServerError::InvalidChannel),
        },
        None => Err(ServerError::InvalidPlatform),
    }
}
