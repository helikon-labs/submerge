use serde::Serialize;
use serde_json::Value as JSONValue;
use utoipa::{ToResponse, ToSchema};

use crate::types::api::dto::pagination::PaginationData;

/// Basic fields to represent a runtime version metadata.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(
    as = MetadataSummary,
    example = metadata_summary_example,
)]
pub struct MetadataDTO {
    /// Metadata runtime spec version.
    #[schema(example = 1001)]
    pub spec_version: u32,
    /// Metadata version.
    #[schema(example = 14)]
    pub metadata_version: u32,
}

#[derive(Debug, Serialize, ToResponse, ToSchema)]
#[response(
    headers(
        ("X-RateLimit-Limit" = u32),
        ("X-RateLimit-Remaining" = u32),
    ),
    description = "Response with a paginated list of metadata summaries.",
)]
pub struct PaginatedMetadataList {
    #[schema(example = json!([metadata_summary_example()]))]
    pub data: Vec<MetadataDTO>,
    pub pagination: PaginationData,
}

fn metadata_summary_example() -> JSONValue {
    let metadata = MetadataDTO {
        spec_version: 1001,
        metadata_version: 14,
    };
    serde_json::to_value(&metadata).unwrap()
}
