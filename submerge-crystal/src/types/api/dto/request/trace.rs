use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

use crate::types::api::{dto::response::hex::HexStringParam, error::APIError};

/// Query parameters for fetching and filtering traces.
#[derive(Debug, Deserialize, IntoParams, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TraceQuery {
    /// Opaque cursor for trace pagination - returned in the endpoint response.
    /// This parameter is mutually exclusive with all other parameters,
    /// and will return bad request if any other parameter is set.
    #[param(required = false, nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Number of traces per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Filter traces by minimum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 1534287)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_block_number: Option<u64>,
    /// Filter traces by maximum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 2825701)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block_number: Option<u64>,
    /// Filter traces by minimum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_block_timestamp: Option<u64>,
    /// Filter traces by maximum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block_timestamp: Option<u64>,
    /// Filter traces by minimum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_spec_version: Option<u32>,
    /// Filter traces by maximum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_spec_version: Option<u32>,
    /// Filter traces by Substrate storage key prefix.
    #[param(required = false, nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_prefix: Option<HexStringParam>,
    /// Filter traces by Substrate storage key parameters.
    #[param(required = false, nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_params: Option<HexStringParam>,
}

impl TraceQuery {
    pub(crate) fn validate_next_cursor_mutually_exclusive(&self) -> Result<(), APIError> {
        if self.next_cursor.is_some() {
            let mut other_fields = Vec::new();
            if self.page_size.is_some() {
                other_fields.push("page_size");
            }
            if self.min_block_number.is_some() {
                other_fields.push("min_block_number");
            }
            if self.max_block_number.is_some() {
                other_fields.push("max_block_number");
            }
            if self.min_block_timestamp.is_some() {
                other_fields.push("min_block_timestamp");
            }
            if self.max_block_timestamp.is_some() {
                other_fields.push("max_block_timestamp");
            }
            if self.min_spec_version.is_some() {
                other_fields.push("min_spec_version");
            }
            if self.max_spec_version.is_some() {
                other_fields.push("max_spec_version");
            }
            if self.key_prefix.is_some() {
                other_fields.push("key_prefix");
            }
            if self.key_params.is_some() {
                other_fields.push("key_params");
            }
            if !other_fields.is_empty() {
                return Err(APIError::BadRequest(
                    format!(
                        "No other parameter should not be set when next_cursor is set. Please remove these parameters and try again: {}",
                        other_fields.join(", ")
                    )
                ));
            }
        }
        Ok(())
    }
}

/// Query parameters for fetching and filtering traces within a block.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub(crate) struct BlockTraceQuery {
    /// Block traces list page number to retrieve. 1-indexed.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        default = 1,
        example = 1
    )]
    pub page: Option<u32>,
    /// Number of traces traces in block per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    pub page_size: Option<u32>,
}
