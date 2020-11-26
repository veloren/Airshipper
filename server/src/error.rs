use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder, Response},
};
use rusoto_s3::{DeleteObjectError, PutObjectError};
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    // Web facing
    #[error("Respond with Status: {0}")]
    Status(Status),

    // Internal errors
    #[error("S3 upload error: {0}")]
    S3UploadError(#[from] rusoto_core::RusotoError<PutObjectError>),
    #[error("S3 delete error: {0}")]
    S3DeleteError(#[from] rusoto_core::RusotoError<DeleteObjectError>),
    #[error("Internal Error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Diesel error: {0}")]
    DieselError(#[from] diesel::result::Error),
    #[error("Internal Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Internal Metrics Error: {0}")]
    PrometheusError(#[from] prometheus::Error),
    #[error("Code '{0}' received with artifact {1:?}")]
    InvalidResponseCode(reqwest::StatusCode, crate::models::Artifact),
}

#[allow(clippy::needless_lifetimes)]
impl<'r, 'o> Responder<'r, 'static> for ServerError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let mut resp = Response::build();

        match self {
            // Web facing errors
            ServerError::Status(status) => {
                resp.status(status);
            },

            // Internal errors
            error => {
                resp.status(Status::InternalServerError);
                let body = format!(
                    "We hit a serious error with your request to '{}'. Please report this to @Songtronix#4790 on \
                     Discord!",
                    req.uri()
                );
                resp.sized_body(body.len(), Cursor::new(body));
                tracing::error!("Internal Error with request[{}]: {}", req, error);
            },
        }

        Ok(resp.finalize())
    }
}

impl From<Status> for ServerError {
    fn from(status: Status) -> Self {
        Self::Status(status)
    }
}
