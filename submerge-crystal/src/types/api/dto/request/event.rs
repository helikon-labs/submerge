use serde::Deserialize;
use utoipa::IntoParams;

/// Query parameters for fetching and filtering events.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub struct EventQuery {
    /// Events list page number to retrieve. 1-indexed.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        default = 1,
        example = 1
    )]
    pub page: Option<u32>,
    /// Number of events per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    pub page_size: Option<u32>,
    /// Filter events by minimum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 1534287)]
    pub min_block_number: Option<u64>,
    /// Filter events by maximum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 2825701)]
    pub max_block_number: Option<u64>,
    /// Filter events by minimum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    pub min_block_timestamp: Option<u64>,
    /// Filter events by maximum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    pub max_block_timestamp: Option<u64>,
    /// Filter events by minimum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    pub min_spec_version: Option<u32>,
    /// Filter events by maximum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    pub max_spec_version: Option<u32>,
    /// Filter events by pallet name. Case insensitive. Stored in `camelCase`.
    #[param(required = false, nullable = false, example = "balances")]
    pub pallet_name: Option<String>,
    /// Filter events by event name. Case insensitive. Stored in `camelCase`.
    #[param(required = false, nullable = false, example = "transfer")]
    pub event_name: Option<String>,
}

/// Query parameters for fetching and filtering events within a block.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub struct BlockEventQuery {
    /// Block event list page number to retrieve. 1-indexed.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        default = 1,
        example = 1
    )]
    pub page: Option<u32>,
    /// Number of block events per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    pub page_size: Option<u32>,
    /// Filter events calls by pallet name. Case insensitive.
    #[param(required = false, nullable = false, example = "staking")]
    pub pallet_name: Option<String>,
    /// Filter blocks events by event name. Case insensitive.
    #[param(required = false, nullable = false, example = "bonded")]
    pub pallet_event_name: Option<String>,
}
