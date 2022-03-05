use crate::{config::Platform, metrics::Metrics, Result};
use rocket::{http::Status, response::Redirect, serde::json::Json, *};
use std::sync::Arc;

// List all channels that are supported for a specific platform
#[get("/channels/<os>/<arch>")]
pub async fn channels(
    _db: crate::DbConnection,
    os: String,
    arch: String,
) -> Json<Vec<String>> {
    let platform = Platform { os, arch };

    let result = crate::CONFIG
        .channels
        .iter()
        .flat_map(|(name, c)| {
            if c.build_map.iter().any(|m| m.platform == platform) {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Json(result)
}

// If no channel specified we default to nightly.
// NOTE: We want to change this behaviour once stable releases are more used than nightly
#[get("/version/<platform>")]
pub async fn version(db: crate::DbConnection, platform: String) -> Result<String> {
    match db.get_latest_version(platform, "nightly").await? {
        Some(ver) => Ok(ver),
        None => Err(Status::NotFound.into()),
    }
}

#[get("/version/<platform>/<channel>")]
pub async fn channel_version(
    db: crate::DbConnection,
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
    db: crate::DbConnection,
    metrics: &State<Arc<Metrics>>,
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
    db: crate::DbConnection,
    metrics: &State<Arc<Metrics>>,
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
