use std::fmt;

#[cfg(feature = "networking")]
use reqwest::Error as ReqwestError;

#[derive(Debug)]
#[allow(clippy::pub_enum_variant_names)]
pub enum TealdeerError {
    CacheError(String),
    ConfigError(String),
    UpdateError(String),
}

#[cfg(feature = "networking")]
impl From<ReqwestError> for TealdeerError {
    fn from(err: ReqwestError) -> Self {
        TealdeerError::UpdateError(format!("HTTP error: {}", err.to_string()))
    }
}

impl fmt::Display for TealdeerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TealdeerError::CacheError(e) => write!(f, "CacheError: {}", e),
            TealdeerError::ConfigError(e) => write!(f, "ConfigError: {}", e),
            TealdeerError::UpdateError(e) => write!(f, "UpdateError: {}", e),
        }
    }
}
