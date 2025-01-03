#![allow(clippy::nonstandard_macro_braces)]
use std::panic;

#[derive(Clone, thiserror::Error, derive_more::Display, Debug)]
pub enum ClientError {
    #[display("Error while performing filesystem operations.")]
    IoError,
    #[display("Error while performing network operations.")]
    NetworkError,
    #[display("FATAL: Failed to start GUI!")]
    IcedError,
    #[display("FATAL: Failed to save/load airshipper configuration!")]
    RonError,
    #[display("Failed to parse Veloren News.")]
    RssError,
    #[display("Failed to open webbrowser.")]
    OpenerError,
    #[display("Error with downloaded veloren archive.")]
    ArchiveError,
    #[display("Error parsing url.")]
    UrlParseError,
    #[display("Error reading input.")]
    ReadlineError,
    #[display("Error parsing image.")]
    ImageError,

    #[cfg(windows)]
    #[display("FATAL: Failed to update airshipper!")]
    UpdateError,
    #[cfg(windows)]
    #[display("Failed to parse version.")]
    VersionError,

    #[display("{}", "_0")]
    Custom(String),
}

impl From<String> for ClientError {
    fn from(err: String) -> Self {
        Self::Custom(err)
    }
}

impl From<rustyline::error::ReadlineError> for ClientError {
    fn from(_: rustyline::error::ReadlineError) -> Self {
        Self::ReadlineError
    }
}

macro_rules! impl_from {
    ($trait:ty, $variant:expr) => {
        impl From<$trait> for ClientError {
            fn from(err: $trait) -> Self {
                tracing::error!("{} => {}", $variant, err);
                $variant
            }
        }
    };
}
impl_from!(std::io::Error, ClientError::IoError);
impl_from!(reqwest::Error, ClientError::NetworkError);
impl_from!(ron::Error, ClientError::RonError);
impl_from!(rss::Error, ClientError::RssError);
impl_from!(opener::OpenError, ClientError::OpenerError);
impl_from!(zip::result::ZipError, ClientError::ArchiveError);
impl_from!(url::ParseError, ClientError::UrlParseError);
impl_from!(iced::Error, ClientError::IcedError);
impl_from!(image::error::ImageError, ClientError::ImageError);
#[cfg(windows)]
impl_from!(self_update::errors::Error, ClientError::UpdateError);
#[cfg(windows)]
impl_from!(semver::Error, ClientError::VersionError);

/// Set up panic handler to relay panics to logs file.
pub fn panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let panic_info_payload = panic_info.payload();
        let payload_string = panic_info_payload.downcast_ref::<String>();
        let reason = match payload_string {
            Some(s) => s.to_string(),
            None => {
                let payload_str = panic_info_payload.downcast_ref::<&str>();
                payload_str.unwrap_or(&"Payload is not a string")
            }
            .to_string(),
        };

        tracing::error!("Airshipper panicked: \n\n{}: {}", reason, panic_info,);

        default_hook(panic_info);
    }));
}
