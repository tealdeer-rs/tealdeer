use reqwest::Error as ReqwestError;
use std::fmt;

#[derive(Debug)]
#[allow(clippy::pub_enum_variant_names)]
pub enum TealdeerError {
    CacheError(String),
    ConfigError(String),
    UpdateError(String),
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
        }
    }
}
