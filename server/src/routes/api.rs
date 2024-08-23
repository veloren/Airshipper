use crate::{
    config::{Platform, API_VERSION},
    db::actions::get_latest_version_uri,
    Context,
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    Json,
};
use serde::Serialize;

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
    // When this is empty return `chrono::Utc::now()` so a client could recheck
    // after a certain time. If there is an actually announcement, choose a static
    // time, e.g. the time you made that announcement public.
    Json(Announcement {
        message: None,
        last_change: chrono::Utc::now(),
    })
}

/// List all channels that are supported for a specific platform
#[tracing::instrument()]
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

#[tracing::instrument(skip(context))]
pub async fn version(
    State(context): State<Context>,
    Path((os, arch, channel)): Path<(String, String, String)>,
) -> impl IntoResponse {
    match get_latest_version_uri(&context.db, &os, &arch, channel).await {
        Ok(Some(vu)) => {
            let version = vu.version;
            tracing::trace!(?version, "serving version");
            (StatusCode::OK, version)
        },
        Ok(None) => {
            tracing::debug!("no version found");
            (StatusCode::NOT_FOUND, "not found".to_string())
        },
        Err(e) => {
            tracing::error!(?e, "Error in /version endpoint");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database error".to_string(),
            )
        },
    }
}

#[tracing::instrument(skip(context))]
pub async fn download(
    State(context): State<Context>,
    Path((os, arch, channel)): Path<(String, String, String)>,
) -> Response<Body> {
    match get_latest_version_uri(&context.db, &os, &arch, &channel).await {
        Ok(Some(vu)) => {
            let uri = vu.uri;
            context.metrics.increment_download(&os, &arch, &channel);
            tracing::trace!(?uri, "serving download location");
            Redirect::to(&uri).into_response()
        },
        Ok(None) => {
            tracing::debug!("no download location found");
            StatusCode::NOT_FOUND.into_response()
        },
        Err(e) => {
            tracing::error!(?e, "Error in /download endpoint");
            (StatusCode::INTERNAL_SERVER_ERROR, "database error").into_response()
        },
    }
}
