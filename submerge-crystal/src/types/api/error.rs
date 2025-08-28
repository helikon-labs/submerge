use std::fmt;

use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

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
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl From<anyhow::Error> for APIError {
    fn from(error: anyhow::Error) -> Self {
        log::error!("API internal server error: {}", error);
        APIError::InternalServerError("Internal server error.".to_string())
    }
}

impl ResponseError for APIError {
    fn error_response(&self) -> HttpResponse {
        match self {
            APIError::BadRequest(_) => HttpResponse::BadRequest().json(Error {
                code: self.error_code(),
                message: self.message(),
            }),
            APIError::InternalServerError(_) => HttpResponse::InternalServerError().json(Error {
                code: self.error_code(),
                message: self.message(),
            }),
            APIError::MetadataNotFound(_) => HttpResponse::NotFound().json(Error {
                code: self.error_code(),
                message: self.message(),
            }),
            APIError::MetadataPalletNotFound(_, _) => HttpResponse::NotFound().json(Error {
                code: self.error_code(),
                message: self.message(),
            }),
        }
    }
}
