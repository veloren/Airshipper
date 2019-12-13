use std::fmt;

pub enum ClientError {
    IoError(std::io::Error),
    LogError(log::SetLoggerError),
    ReqwestError(reqwest::Error),
    ZipError(zip::result::ZipError),
    StripPrefixError(std::path::StripPrefixError),
    Custom(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(x) => write!(f, "{}", x),
            Self::LogError(_) => unreachable!(),
            Self::ReqwestError(x) => write!(f, "{}", x),
            Self::ZipError(x) => write!(f, "{}", x),
            Self::StripPrefixError(x) => {
                write!(f, "Failed to convert absolute to relative path: {}", x)
            }
            Self::Custom(x) => write!(f, "{}", x),
        }
    }
}

impl From<std::io::Error> for ClientError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<log::SetLoggerError> for ClientError {
    fn from(error: log::SetLoggerError) -> Self {
        Self::LogError(error)
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        Self::ReqwestError(error)
    }
}

impl From<String> for ClientError {
    fn from(error: String) -> Self {
        Self::Custom(error)
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
