use std::panic;

#[derive(thiserror::Error, derive_more::Display, Debug)]
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

    #[cfg(windows)]
    #[display("FATAL: Failed to update airshipper!")]
    UpdateError,
    #[cfg(windows)]
    #[display("Failed to parse version.")]
    VersionError,

    #[display("{0}")]
    Custom(String),
}

impl From<String> for ClientError {
    fn from(err: String) -> Self {
        Self::Custom(err)
    }
}

macro_rules! impl_from {
    ($trait:ty, $variant:expr) => {
        impl From<$trait> for ClientError {
            fn from(err: $trait) -> Self {
                log::error!("{} => {}", $variant, err);
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
#[cfg(windows)]
impl_from!(self_update::errors::Error, ClientError::UpdateError);
#[cfg(windows)]
impl_from!(semver::SemVerError, ClientError::VersionError);

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
