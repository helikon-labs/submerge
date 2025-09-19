use std::fmt;

use serde::Serialize;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Clone, Debug, Serialize)]
pub struct Error {
    pub code: u16,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub enum APIError {
    BadRequest(String),
    InternalServerError(String),
    MetadataNotFound(u32),
    MetadataPalletNotFound(u32, u32),
}

impl APIError {
    fn error_code(&self) -> u16 {
        match self {
            APIError::BadRequest(_) => 0,
            APIError::InternalServerError(_) => 1,
            APIError::MetadataNotFound(_) => 2,
            APIError::MetadataPalletNotFound(_, _) => 3,
        }
    }

    fn message(&self) -> String {
        match self {
            APIError::BadRequest(message) => message.to_owned(),
            APIError::InternalServerError(message) => message.to_owned(),
            APIError::MetadataNotFound(spec_version) => {
                format!("Metadata for spec version {} not found.", spec_version)
            }
            APIError::MetadataPalletNotFound(spec_version, index) => format!(
                "Pallet index {} not found in metadata for spec version {}.",
                index, spec_version
            ),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            APIError::BadRequest(_) => StatusCode::BAD_REQUEST,
            APIError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            APIError::MetadataNotFound(_) => StatusCode::NOT_FOUND,
            APIError::MetadataPalletNotFound(_, _) => StatusCode::NOT_FOUND,
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
        let error_response = Error {
            code: self.error_code(),
            message: self.message(),
        };

        (self.status_code(), Json(error_response)).into_response()
    }
}
