use std::io::Error as IoError;

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

impl From<IoError> for TealdeerError {
    fn from(err: IoError) -> TealdeerError {
        TealdeerError::ConfigError(format!("Io error: {}", err))
    }
}
