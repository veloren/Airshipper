#![allow(unused_imports)]
use crate::{
    config::{Platform, API_VERSION},
    db::actions::get_latest_version_uri,
    metrics::Metrics,
    Context, Result,
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header::LOCATION, Response, StatusCode},
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct Version {
    version: u32,
}

/// List the supported version of this REST-API
pub async fn api_version() -> Json<Version> {
    Json(Version {
        version: API_VERSION,
    })
}

#[derive(Serialize)]
pub struct Announcement {
    message: Option<String>,
    last_change: chrono::DateTime<chrono::Utc>,
}

/// Public Service Announcement to be displayed in Airshipper
pub async fn announcement() -> Json<Announcement> {
    use chrono::TimeZone;
    Json(Announcement {
        message: None,
        last_change: chrono::Utc::now(),
    })
}

/// List all channels that are supported for a specific platform
pub async fn channels(Path((os, arch)): Path<(String, String)>) -> Json<Vec<String>> {
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

pub async fn version(
    State(context): State<Context>,
    Path((os, arch, channel)): Path<(String, String, String)>,
) -> impl IntoResponse {
    match get_latest_version_uri(&context.db, &os, &arch, channel).await {
        Ok(Some(vu)) => (StatusCode::OK, vu.version),
        Ok(None) => (StatusCode::NOT_FOUND, "not found".to_string()),
        Err(e) => {
            tracing::error!(?e, "Error in /version endpoint");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database error".to_string(),
            )
        },
    }
}

pub async fn download(
    State(context): State<Context>,
    Path((os, arch, channel)): Path<(String, String, String)>,
) -> Response<Body> {
    tracing::info!(?os, ?arch, ?channel, "Serving Download");
    match get_latest_version_uri(&context.db, &os, &arch, &channel).await {
        Ok(Some(vu)) => {
            context.metrics.increment_download(&os, &arch, &channel);
            Response::builder()
                .status(303)
                .header(LOCATION, vu.uri)
                .body(Body::empty())
                .unwrap()
        },
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(?e, "Error in /download endpoint");
            (StatusCode::INTERNAL_SERVER_ERROR, "database error").into_response()
        },
    }
}
