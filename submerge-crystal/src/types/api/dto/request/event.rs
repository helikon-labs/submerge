use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

/// Query parameters for fetching and filtering events.
#[derive(Debug, Serialize, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub(crate) struct EventQuery {
    /// Opaque cursor for pagination. If provided, all filter params are ignored.
    #[param(required = false, nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Number of events per page to be returned.
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
    /// Filter events by minimum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 1534287)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_block_number: Option<u64>,
    /// Filter events by maximum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 2825701)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block_number: Option<u64>,
    /// Filter events by minimum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_block_timestamp: Option<u64>,
    /// Filter events by maximum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block_timestamp: Option<u64>,
    /// Filter events by minimum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_spec_version: Option<u32>,
    /// Filter events by maximum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_spec_version: Option<u32>,
    /// Filter events by pallet name. Case insensitive. Stored in `camelCase`.
    #[param(required = false, nullable = false, example = "balances")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pallet_name: Option<String>,
    /// Filter events by event name. Case insensitive. Stored in `camelCase`.
    #[param(required = false, nullable = false, example = "transfer")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_name: Option<String>,
    /// Whether to include event arguments in the events in the response.
    /// Default is `false`. Setting this to `true` increases response size considerably.
    /// Prefer to use the `GET /event/{event_hash}/args` endpoint per event instead.
    #[serde(default)]
    #[param(required = false, default = false)]
    pub include_args: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct EventCursorPosition {
    pub(crate) block_number: u64,
    pub(crate) block_hash_hex: String,
    pub(crate) index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct EventCursorPayload {
    pub(crate) cursor_position: EventCursorPosition,
    pub(crate) query: EventQuery,
}

/// Query parameters for fetching and filtering events within a block.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub(crate) struct BlockEventQuery {
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
    /// Whether to include event arguments in the block events in the response.
    /// Default is `false`. Setting this to `true` increases response size considerably.
    /// Prefer to use the `GET /event/{event_hash}/args` endpoint per call instead.
    #[serde(default)]
    #[param(required = false, default = false)]
    pub include_args: bool,
}

/// Query parameter for definining whether the endpoint should include event arguments
/// within the returned event(s).
#[derive(Debug, Deserialize, IntoParams)]
pub(crate) struct IncludeEventArgsParam {
    /// Whether to include event arguments in the event(s) in the response.
    /// Default is `false`. Setting this to `true` increases the response size considerably.
    /// Prefer to use the `GET /event/{event_hash}/args` endpoint per call instead.
    #[serde(default)]
    #[param(required = false, default = false)]
    pub include_args: bool,
}
