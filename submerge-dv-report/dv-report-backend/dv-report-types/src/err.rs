//! Error types.
use serde::{Deserialize, Serialize};
use sp_core::bytes::FromHexError;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ServiceError {
    pub description: String,
}

impl ServiceError {
    pub fn from(description: &str) -> ServiceError {
        ServiceError {
            description: description.to_string(),
        }
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum DecodeError {
    #[error("Decode error: {0}")]
    Error(String),
}

impl From<FromHexError> for DecodeError {
    fn from(error: FromHexError) -> Self {
        Self::Error(error.to_string())
    }
}

impl From<parity_scale_codec::Error> for DecodeError {
    fn from(error: parity_scale_codec::Error) -> Self {
        Self::Error(error.to_string())
    }
}
