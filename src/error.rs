use std::fmt;

use reqwest::Error as ReqwestError;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum TealdeerError {
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
            Self::UpdateError(e) => write!(f, "UpdateError: {}", e),
        }
    }
}
