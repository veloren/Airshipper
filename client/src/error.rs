use std::panic;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("A network error occured: {0}")]
    NetworkError(#[from] reqwest::Error),
    // hopefully rare errors
    #[error("FATAL: Failed to save state: {0}")]
    SerializeError(#[from] ron::ser::Error),
    #[error("FATAL: Failed to load state: {0}")]
    DeserializeError(#[from] ron::de::Error),
    #[error("Failed to parse News: {0}")]
    RssError(#[from] rss::Error),
    #[error("Failed to open browser: {0}")]
    OpenerError(#[from] opener::OpenError),
    #[error("Error with archive: {0}")]
    ArchiveError(#[from] zip::result::ZipError),
    #[error("Error parsing url: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[cfg(windows)]
    #[error("Failed to update myself: {0}")]
    UpdateError(#[from] self_update::errors::Error),
    #[cfg(windows)]
    #[error("Failed to parse version: {0}")]
    VersionError(#[from] semver::SemVerError),

    #[error("{0}")]
    Custom(String),
}

impl From<String> for ClientError {
    fn from(err: String) -> Self {
        Self::Custom(err)
    }
}

/// Set up panic handler to relay panics to logs file.
pub fn panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let panic_info_payload = panic_info.payload();
        let payload_string = panic_info_payload.downcast_ref::<String>();
        let reason = match payload_string {
            Some(s) => &s,
            None => {
                let payload_str = panic_info_payload.downcast_ref::<&str>();
                match payload_str {
                    Some(st) => st,
                    None => "Payload is not a string",
                }
            },
        };

        log::error!("Airshipper panicked: \n\n{}: {}", reason, panic_info,);

        default_hook(panic_info);
    }));
}
