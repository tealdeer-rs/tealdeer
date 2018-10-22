use std::fmt;
use curl::Error as CurlError;

#[derive(Debug)]
pub enum TealdeerError {
    CacheError(String),
    ConfigError(String),
    UpdateError(String),
}

impl From<CurlError> for TealdeerError {
    fn from(err: CurlError) -> TealdeerError {
        TealdeerError::UpdateError(format!("Curl error: {}", err.to_string()))
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
