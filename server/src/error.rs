use std::io::Cursor;

use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder, Response},
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    // Web facing
    #[error("Invalid platform. Currently supported are windows and linux.")]
    InvalidPlatform,
    #[error("Invalid channel. Currently supported is nightly with upcoming support for releases.")]
    InvalidChannel,
    // Not really a serious error (see routes/api.rs)
    #[error("Respond with Status: {0}")]
    Status(Status),

    // Internal errors
    #[error("S3Bucker error: {0}")]
    S3Bucket(#[from] s3::error::S3Error),
    #[error("Internal Error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Diesel error: {0}")]
    DieselError(#[from] diesel::result::Error),
    #[error("Internal Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Code '{0}' received when requesting artifact {1:?}")]
    InvalidResponseCode(reqwest::StatusCode, crate::models::Artifact),
}

#[rocket::async_trait]
impl<'r> Responder<'r> for ServerError {
    async fn respond_to(self, req: &'r Request<'_>) -> response::Result<'r> {
        let mut resp = Response::build();

        match self {
            // Web facing errors
            ServerError::InvalidPlatform => {
                resp.status(Status::BadRequest);
                resp.sized_body(Cursor::new(format!(
                    "Invalid platform. Currently supported are windows and linux."
                )))
                .await; // TODO: Do not hardcode (use enum_iterator or such)
            },
            ServerError::InvalidChannel => {
                resp.status(Status::BadRequest);
                resp.sized_body(Cursor::new(format!(
                    "Invalid channel. Currently supported is nightly with upcoming support for releases."
                )))
                .await; // TODO: Do not hardcode (use enum_iterator or such)
            },
            ServerError::Status(status) => {
                resp.status(status).finalize();
            },

            // Internal errors
            error => {
                resp.status(Status::InternalServerError);
                resp.sized_body(Cursor::new(format!(
                    "We hit a serious error with your request to '{}'. Please report this to @Songtronix#4790 on \
                     Discord!",
                    req.uri()
                )))
                .await;
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
