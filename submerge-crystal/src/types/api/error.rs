use std::{fmt, string};

use serde::Serialize;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use utoipa::ToSchema;

/// Generic error type.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[schema(as = Error)]
pub(crate) struct APIErrorBody {
    /// Error message.
    #[schema(format = "text", example = "Error message.")]
    pub message: String,
}

#[derive(Clone, Debug, ToSchema)]
pub(crate) enum APIError {
    NotFound,
    SerializationError,
    BadRequest(String),
    InternalServerError(String),
    MetadataNotFound(u32),
    MetadataPalletNotFound(u32, u32),
    BlockNotFoundWithNumber(u64),
    BlockNotFoundWithHash(Vec<u8>),
    BlockExtrinsicNotFoundWithNumberAndIndex(u64, u32),
    BlockExtrinsicNotFoundWithHashAndIndex(Vec<u8>, u32),
    ExtrinsicNotFoundWithHash(Vec<u8>),
    CallNotFoundWithHash(Vec<u8>),
    EventNotFoundWithHash(Vec<u8>),
    TraceNotFoundWithHash(Vec<u8>),
    ParentCallNotFoundForCallWithHash(Vec<u8>),
    InvalidHex(String),
    InvalidUTF8(String),
    InvalidBlockAuthor(String),
    InvalidExtrinsicSigner(String),
    JSONError(String),
}

impl APIError {
    fn message(&self) -> String {
        match self {
            APIError::NotFound => "Not found.".to_owned(),
            APIError::SerializationError => "Serialization error.".to_owned(),
            APIError::BadRequest(message) => message.to_owned(),
            APIError::InternalServerError(message) => message.to_owned(),
            APIError::MetadataNotFound(spec_version) => {
                format!("Metadata for spec version {spec_version} not found.")
            }
            APIError::MetadataPalletNotFound(spec_version, index) => format!(
                "Pallet index {index} not found in metadata for spec version {spec_version}.",
            ),
            APIError::BlockNotFoundWithNumber(number) => {
                format!("Block with number {number} not found.",)
            }
            APIError::BlockNotFoundWithHash(hash) => {
                format!("Block with hash 0x{} not found.", hex::encode(hash))
            }
            APIError::BlockExtrinsicNotFoundWithNumberAndIndex(block_number, extrinsic_index) => {
                format!(
                    "Extrinsic with index {} not found in block(s) with number {}.",
                    extrinsic_index, block_number
                )
            }
            APIError::BlockExtrinsicNotFoundWithHashAndIndex(block_hash, extrinsic_index) => {
                format!(
                    "Extrinsic with index {} not found in block with hash 0x{}.",
                    extrinsic_index,
                    hex::encode(block_hash)
                )
            }
            APIError::ExtrinsicNotFoundWithHash(hash) => {
                format!("Extrinsic with hash 0x{} not found.", hex::encode(hash))
            }
            APIError::CallNotFoundWithHash(hash) => {
                format!("Call with hash 0x{} not found.", hex::encode(hash))
            }
            APIError::EventNotFoundWithHash(hash) => {
                format!("Event with hash 0x{} not found.", hex::encode(hash))
            }
            APIError::TraceNotFoundWithHash(hash) => {
                format!("Trace with hash 0x{} not found.", hex::encode(hash))
            }
            APIError::ParentCallNotFoundForCallWithHash(hash) => {
                format!(
                    "Call with hash 0x{} does not have a parent call.",
                    hex::encode(hash)
                )
            }
            APIError::InvalidHex(error) => error.to_string(),
            APIError::InvalidUTF8(error) => error.to_string(),
            APIError::InvalidBlockAuthor(author) => {
                format!("Invalid block author: {author}. Enter valid SS58 address or hexadecimal string.")
            }
            APIError::InvalidExtrinsicSigner(author) => {
                format!("Invalid extrinsic signer: {author}. Enter valid SS58 address or hexadecimal string.")
            }
            APIError::JSONError(error) => {
                format!("JSON error: {error}")
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
            APIError::BlockExtrinsicNotFoundWithNumberAndIndex(_, _) => StatusCode::NOT_FOUND,
            APIError::BlockExtrinsicNotFoundWithHashAndIndex(_, _) => StatusCode::NOT_FOUND,
            APIError::ExtrinsicNotFoundWithHash(_) => StatusCode::NOT_FOUND,
            APIError::CallNotFoundWithHash(_) => StatusCode::NOT_FOUND,
            APIError::EventNotFoundWithHash(_) => StatusCode::NOT_FOUND,
            APIError::TraceNotFoundWithHash(_) => StatusCode::NOT_FOUND,
            APIError::ParentCallNotFoundForCallWithHash(_) => StatusCode::NOT_FOUND,
            APIError::InvalidHex(_) => StatusCode::BAD_REQUEST,
            APIError::InvalidUTF8(_) => StatusCode::BAD_REQUEST,
            APIError::InvalidBlockAuthor(_) => StatusCode::BAD_REQUEST,
            APIError::InvalidExtrinsicSigner(_) => StatusCode::BAD_REQUEST,
            APIError::JSONError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
        tracing::error!("API internal server error: {}", error);
        APIError::InternalServerError("Internal server error.".to_string())
    }
}

impl From<hex::FromHexError> for APIError {
    fn from(error: hex::FromHexError) -> Self {
        tracing::error!("Hexadecimal decode error: {}", error);
        APIError::InvalidHex(error.to_string())
    }
}

impl From<string::FromUtf8Error> for APIError {
    fn from(error: string::FromUtf8Error) -> Self {
        tracing::error!("UTF-8 conversion error: {}", error);
        APIError::InvalidUTF8(error.to_string())
    }
}

impl From<serde_json::Error> for APIError {
    fn from(error: serde_json::Error) -> Self {
        tracing::error!("Serde JSON error: {}", error);
        APIError::JSONError(error.to_string())
    }
}

impl From<base64::DecodeError> for APIError {
    fn from(error: base64::DecodeError) -> Self {
        tracing::error!("Base64 decode error: {}", error);
        APIError::BadRequest(error.to_string())
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
