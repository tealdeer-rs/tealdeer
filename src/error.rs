use reqwest::Error as ReqwestError;
use std::fmt;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum TealdeerError {
    CacheError(String),
    ConfigError(String),
    UpdateError(String),
    WriteError(String),
}

impl TealdeerError {
    pub fn message(&self) -> &str {
        match self {
            Self::CacheError(msg)
            | Self::ConfigError(msg)
            | Self::UpdateError(msg)
            | Self::WriteError(msg) => msg,
        }
    }
}

impl From<ReqwestError> for TealdeerError {
    fn from(err: ReqwestError) -> Self {
        Self::UpdateError(format!("HTTP error: {}", err.to_string()))
    }
}

impl fmt::Display for TealdeerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CacheError(e) => write!(f, "CacheError: {}", e),
            Self::ConfigError(e) => write!(f, "ConfigError: {}", e),
            Self::UpdateError(e) => write!(f, "UpdateError: {}", e),
            Self::WriteError(e) => write!(f, "WriteError: {}", e),
        }
    }
}
