use serde::Deserialize;

use crate::types::api::dto::pagination::PaginationQuery;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub min_block_number: Option<u64>,
    pub max_block_number: Option<u64>,
    pub min_block_timestamp: Option<u64>,
    pub max_block_timestamp: Option<u64>,
    pub min_spec_version: Option<u64>,
    pub max_spec_version: Option<u64>,
    pub pallet_name: Option<String>,
    pub pallet_event_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockEventQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub pallet_name: Option<String>,
    pub pallet_event_name: Option<String>,
}