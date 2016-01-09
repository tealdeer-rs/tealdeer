use curl::ErrCode;


#[derive(Debug)]
pub enum TldrError {
    CacheError(String),
    UpdateError(String),
}


impl From<ErrCode> for TldrError {
    fn from(err: ErrCode) -> TldrError {
        TldrError::UpdateError(err.to_string())
    }
}
