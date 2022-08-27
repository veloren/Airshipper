use crate::{
    config::{Platform, API_VERSION},
    metrics::Metrics,
    Result,
};
use rocket::{http::Status, response::Redirect, serde::json::Json, *};
use std::sync::Arc;

#[derive(crate::serde::Serialize)]
pub struct Version {
    version: u32,
}

// List all channels that are supported for a specific platform
#[get("/api/version")]
pub async fn api_version(_db: crate::DbConnection) -> Json<Version> {
    Json(Version {
        version: API_VERSION,
    })
}

#[derive(crate::serde::Serialize)]
pub struct Announcement {
    message: Option<String>,
    last_change: chrono::DateTime<chrono::Utc>,
}

// List all channels that are supported for a specific platform
#[get("/announcement")]
pub async fn announcement(_db: crate::DbConnection) -> Json<Announcement> {
    Json(Announcement {
        message: None,
        last_change: chrono::Utc::now(),
    })
}

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

#[get("/version/<os>/<arch>/<channel>")]
pub async fn version(
    db: crate::DbConnection,
    os: String,
    arch: String,
    channel: String,
) -> Result<String> {
    match db.get_latest_version(&os, &arch, channel).await? {
        Some(ver) => Ok(ver),
        None => Err(Status::NotFound.into()),
    }
}

#[get("/latest/<os>/<arch>/<channel>")]
pub async fn download(
    db: crate::DbConnection,
    metrics: &State<Arc<Metrics>>,
    os: String,
    arch: String,
    channel: String,
) -> Result<Redirect> {
    match db.get_latest_uri(&os, &arch, &channel).await? {
        Some(uri) => {
            metrics.increment_download(&os, &arch, &channel);
            Ok(Redirect::to(uri))
        },
        None => Err(Status::NotFound.into()),
    }
}
