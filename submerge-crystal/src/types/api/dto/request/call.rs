use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

use crate::types::api::error::APIError;

/// Query parameters for fetching and filtering calls.
#[derive(Debug, Deserialize, IntoParams, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CallQuery {
    /// Opaque cursor for call pagination - returned in the endpoint response.
    /// This parameter is mutually exclusive with all other parameters,
    /// and will return bad request if any other parameter is set.
    #[param(required = false, nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Number of calls per page to be returned.
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
    /// Filter calls by minimum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 1534287)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_block_number: Option<u64>,
    /// Filter calls by maximum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 2825701)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block_number: Option<u64>,
    /// Filter calls by minimum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_block_timestamp: Option<u64>,
    /// Filter calls by maximum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block_timestamp: Option<u64>,
    /// Filter calls by minimum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_spec_version: Option<u32>,
    /// Filter calls by maximum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_spec_version: Option<u32>,
    /// Filter calls by pallet name. Case insensitive. Stored in `camelCase`.
    #[param(required = false, nullable = false, example = "system")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_name: Option<String>,
    /// Filter calls by call name. Case insensitive. Stored in `camelCase`.
    #[param(required = false, nullable = false, example = "setcode")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_name: Option<String>,
    /// Whether to include only calls within signed/unsigned extrinsics.
    #[param(required = false, nullable = false, example = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_signed: Option<bool>,
    /// Whether to include call arguments in the calls in the response.
    /// Default is `false`. Setting this to `true` increases the response size considerably.
    /// Prefer to use the `GET /call/{call_hash}/args` endpoint per call instead.
    #[serde(default)]
    #[param(required = false, default = false)]
    pub include_args: bool,
}

impl CallQuery {
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
            if self.pallet_name.is_some() {
                other_fields.push("pallet_name");
            }
            if self.call_name.is_some() {
                other_fields.push("call_name");
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

/// Query parameters for fetching and filtering calls within a block.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub(crate) struct BlockCallQuery {
    /// Block call list page number to retrieve. 1-indexed.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        default = 1,
        example = 1
    )]
    pub page: Option<u32>,
    /// Number of block calls per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    pub page_size: Option<u32>,
    /// Filter blocks calls by pallet name. Case insensitive.
    #[param(required = false, nullable = false, example = "system")]
    pub pallet_name: Option<String>,
    /// Filter blocks calls by call name. Case insensitive.
    #[param(required = false, nullable = false, example = "setcode")]
    pub call_name: Option<String>,
    /// Whether to include only calls within signed/unsigned extrinsics.
    #[param(required = false, nullable = false, example = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_signed: Option<bool>,
    /// Whether to include call arguments in the block calls in the response.
    /// Default is `false`. Setting this to `true` increases the response size considerably.
    /// Prefer to use the `GET /call/{call_hash}/args` endpoint per call instead.
    #[serde(default)]
    #[param(required = false, default = false)]
    pub include_args: bool,
}

/// Query parameter for definining whether the endpoint should include call arguments
/// within the returned call(s).
#[derive(Debug, Deserialize, IntoParams)]
pub(crate) struct IncludeCallArgsParam {
    /// Whether to include call arguments in the call(s) in the response.
    /// Default is `false`. Setting this to `true` increases the response size considerably.
    /// Prefer to use the `GET /call/{call_hash}/args` endpoint per call instead.
    #[serde(default)]
    #[param(required = false, default = false)]
    pub include_args: bool,
}
