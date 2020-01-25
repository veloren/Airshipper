use std::fmt;

#[derive(Debug)]
pub enum ClientError {
    IoError(std::io::Error),
    NetworkError(isahc::Error),
    Custom(String),
    // Should hopefully never occur
    RssError(rss::Error),
    ZipError(zip::result::ZipError),
    LogError(log::SetLoggerError),
    StripPrefixError(std::path::StripPrefixError),
    HttpError(isahc::http::Error),
    SerializeError(ron::ser::Error),
    DeserializeError(ron::de::Error),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(x) => write!(f, "{}", x),
            Self::NetworkError(x) => write!(f, "{}", x),
            Self::Custom(x) => write!(f, "{}", x),

            Self::RssError(x) => write!(f, "Failed parsing news: {}", x),
            Self::ZipError(x) => write!(f, "{}", x),
            Self::StripPrefixError(x) => {
                write!(f, "Failed to convert absolute to relative path: {}", x)
            }
            Self::LogError(_) => unreachable!(),
            Self::SerializeError(x) => write!(f, "FATAL: Failed to save the config! {}", x),
            Self::DeserializeError(x) => write!(f, "FATAL: Failed to load the config! {}", x),
            Self::HttpError(x) => write!(f, "{}", x),
        }
    }
}

impl From<std::io::Error> for ClientError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<isahc::Error> for ClientError {
    fn from(error: isahc::Error) -> Self {
        Self::NetworkError(error)
    }
}

impl From<rss::Error> for ClientError {
    fn from(error: rss::Error) -> Self {
        Self::RssError(error)
    }
}

impl From<isahc::http::Error> for ClientError {
    fn from(error: isahc::http::Error) -> Self {
        Self::HttpError(error)
    }
}

impl From<log::SetLoggerError> for ClientError {
    fn from(error: log::SetLoggerError) -> Self {
        Self::LogError(error)
    }
}

impl From<String> for ClientError {
    fn from(error: String) -> Self {
        Self::Custom(error)
    }
}

impl From<&str> for ClientError {
    fn from(error: &str) -> Self {
        Self::Custom(error.into())
    }
}

impl From<zip::result::ZipError> for ClientError {
    fn from(error: zip::result::ZipError) -> Self {
        Self::ZipError(error)
    }
}

impl From<std::path::StripPrefixError> for ClientError {
    fn from(error: std::path::StripPrefixError) -> Self {
        Self::StripPrefixError(error)
    }
}

impl From<ron::ser::Error> for ClientError {
    fn from(error: ron::ser::Error) -> Self {
        Self::SerializeError(error)
    }
}

impl From<ron::de::Error> for ClientError {
    fn from(error: ron::de::Error) -> Self {
        Self::DeserializeError(error)
    }
}
