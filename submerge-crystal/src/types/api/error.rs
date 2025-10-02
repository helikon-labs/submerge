use std::fmt;

use serde::Serialize;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Clone, Debug, Serialize)]
pub struct APIErrorBody {
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub enum APIError {
    NotFound,
    SerializationError,
    BadRequest(String),
    InternalServerError(String),
    MetadataNotFound(u32),
    MetadataPalletNotFound(u32, u32),
    BlockNotFoundWithNumber(u64),
    BlockNotFoundWithHash(Vec<u8>),
}

impl APIError {
    fn message(&self) -> String {
        match self {
            APIError::NotFound => "Not found.".to_owned(),
            APIError::SerializationError => "Serialization error.".to_owned(),
            APIError::BadRequest(message) => message.to_owned(),
            APIError::InternalServerError(message) => message.to_owned(),
            APIError::MetadataNotFound(spec_version) => {
                format!("Metadata for spec version {} not found.", spec_version)
            }
            APIError::MetadataPalletNotFound(spec_version, index) => format!(
                "Pallet index {} not found in metadata for spec version {}.",
                index, spec_version,
            ),
            APIError::BlockNotFoundWithNumber(number) => {
                format!("Block with number {} not found.", number,)
            }
            APIError::BlockNotFoundWithHash(hash) => {
                format!("Block with hash 0x{} not found.", hex::encode(hash),)
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            APIError::NotFound => StatusCode::NOT_FOUND,
            APIError::SerializationError => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::BadRequest(_) => StatusCode::BAD_REQUEST,
            APIError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::MetadataNotFound(_) => StatusCode::NOT_FOUND,
            APIError::MetadataPalletNotFound(_, _) => StatusCode::NOT_FOUND,
            APIError::BlockNotFoundWithNumber(_) => StatusCode::NOT_FOUND,
            APIError::BlockNotFoundWithHash(_) => StatusCode::NOT_FOUND,
        }
    }
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for APIError {}

impl From<anyhow::Error> for APIError {
    fn from(error: anyhow::Error) -> Self {
        log::error!("API internal server error: {}", error);
        APIError::InternalServerError("Internal server error.".to_string())
    }
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        let body = APIErrorBody {
            message: self.message(),
        };

        (self.status_code(), Json(body)).into_response()
    }
}
