use serde::Deserialize;
use utoipa::IntoParams;

use crate::types::api::dto::response::hex::HexStringParam;

/// Query parameters for fetching and filtering traces.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub struct TraceQuery {
    /// Trace list page number to retrieve. 1-indexed.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        default = 1,
        example = 1
    )]
    pub page: Option<u32>,
    /// Number of traces per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    pub page_size: Option<u32>,
    /// Filter traces by minimum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 1534287)]
    pub min_block_number: Option<u64>,
    /// Filter traces by maximum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 2825701)]
    pub max_block_number: Option<u64>,
    /// Filter traces by minimum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    pub min_block_timestamp: Option<u64>,
    /// Filter traces by maximum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    pub max_block_timestamp: Option<u64>,
    /// Filter traces by minimum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    pub min_spec_version: Option<u32>,
    /// Filter traces by maximum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    pub max_spec_version: Option<u32>,
    /// Filter traces by Substrate storage key prefix.
    #[param(required = false, nullable = false)]
    pub key_prefix: Option<HexStringParam>,
    /// Filter traces by Substrate storage key parameters.
    #[param(required = false, nullable = false)]
    pub key_params: Option<HexStringParam>,
}

/// Query parameters for fetching and filtering traces within a block.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub struct BlockTraceQuery {
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
