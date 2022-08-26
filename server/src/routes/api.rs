use crate::{config::Platform, config::SUPPORTED_AIRSHIPPER_CLIENT_VERSIONS, metrics::Metrics, Result};
use rocket::{http::Status, response::Redirect, serde::json::Json, *};
use std::sync::Arc;


// List all channels that are supported for a specific platform
#[get("/supported-airshipper-client-versions")]
pub async fn supported_airshipper_client_versions(
    _db: crate::DbConnection,
) -> String {
    SUPPORTED_AIRSHIPPER_CLIENT_VERSIONS.to_string()
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
