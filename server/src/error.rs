use thiserror::Error;

/// for clients of airshipper asking us stuff
#[derive(Error, Debug)]
pub(crate) enum ServerError {
    #[error("Sqlx error: {0}")]
    Database(#[from] sqlx::Error),
}

/// for proceeses triggered internally, e.g. upload of new versions triggered by gitlab
#[derive(Error, Debug)]
pub(crate) enum ProcessError {
    #[error("Internal Error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Sqlx error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Internal Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Octocrab error: {0}")]
    Octocrab(#[from] octocrab::Error),
    #[error("Url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}
