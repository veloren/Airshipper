use crate::{metrics::Metrics, Result};
use rocket::{http::Status, response::Redirect, *};

// If no channel specified we default to nightly.
// NOTE: We want to change this behaviour once stable releases are more used than nightly
#[get("/version/<platform>")]
pub async fn version(
    db: State<'_, crate::DbConnection>,
    platform: String,
) -> Result<String> {
    match db.get_latest_version(platform, "nightly").await? {
        Some(ver) => Ok(ver),
        None => Err(Status::NotFound.into()),
    }
}

#[get("/version/<platform>/<channel>")]
pub async fn channel_version(
    db: State<'_, crate::DbConnection>,
    platform: String,
    channel: String,
) -> Result<String> {
    match db.get_latest_version(platform, channel).await? {
        Some(ver) => Ok(ver),
        None => Err(Status::NotFound.into()),
    }
}

// If no channel specified we default to nightly.
// NOTE: We want to change this behaviour once stable releases are more used than nightly
#[get("/latest/<platform>")]
pub async fn download(
    db: State<'_, crate::DbConnection>,
    metrics: State<'_, Metrics>,
    platform: String,
) -> Result<Redirect> {
    match db.get_latest_uri(&platform, "nightly").await? {
        Some(uri) => {
            metrics.increment(&platform);
            Ok(Redirect::to(uri))
        },
        None => Err(Status::NotFound.into()),
    }
}

#[get("/latest/<platform>/<channel>")]
pub async fn channel_download(
    db: State<'_, crate::DbConnection>,
    metrics: State<'_, Metrics>,
    platform: String,
    channel: String,
) -> Result<Redirect> {
    match db.get_latest_uri(&platform, channel).await? {
        Some(uri) => {
            metrics.increment(&platform);
            Ok(Redirect::to(uri))
        },
        None => Err(Status::NotFound.into()),
    }
}
