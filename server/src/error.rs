use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    // Web facing
    #[error("Respond with Status: {0}")]
    Status(#[from] axum::Error),
    // Internal errors
    #[error("Internal Error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Sqlx error: {0}")]
    DieselError(#[from] sqlx::Error),
    #[error("Internal Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Internal Metrics Error: {0}")]
    PrometheusError(#[from] prometheus::Error),
    #[error("Code '{0}' received with artifact {1:?}")]
    InvalidResponseCode(reqwest::StatusCode, crate::models::Artifact),
    #[error("Octocrab error: {0}")]
    OctocrabError(#[from] octocrab::Error),
    #[error("Url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
}
