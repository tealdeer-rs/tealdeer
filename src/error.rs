use curl::ErrCode;

#[derive(Debug)]
pub enum TealdeerError {
    CacheError(String),
    UpdateError(String),
}

impl From<ErrCode> for TealdeerError {
    fn from(err: ErrCode) -> TealdeerError {
        TealdeerError::UpdateError(err.to_string())
    }
}
