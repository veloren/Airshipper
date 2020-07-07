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
    #[error("Respond with Status: {0}")]
    Status(Status),

    // Internal errors
    #[error("S3Bucket error: {0}")]
    S3Bucket(#[from] s3::S3Error),
    #[error("S3CredentialsBucket error: {0}")]
    CredentialsBucket(#[from] awscreds::AwsCredsError),
    #[error("Internal Error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Diesel error: {0}")]
    DieselError(#[from] diesel::result::Error),
    #[error("Internal Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Code '{0}' received with artifact {1:?}")]
    InvalidResponseCode(reqwest::StatusCode, crate::models::Artifact),
}

#[allow(clippy::needless_lifetimes)]
#[rocket::async_trait]
impl<'r> Responder<'r> for ServerError {
    async fn respond_to(self, req: &'r Request<'_>) -> response::Result<'r> {
        let mut resp = Response::build();

        match self {
            // Web facing errors
            ServerError::Status(status) => {
                resp.status(status);
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
