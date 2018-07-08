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
