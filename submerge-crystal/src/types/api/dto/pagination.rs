use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct PaginationQuery {
    /// Page number to retrieve. 1-indexed.
    #[param(required = false, nullable = false, example = 1)]
    pub page: Option<u32>,
    /// Number of items per page to be returned.
    #[param(required = false, nullable = false, example = 100)]
    pub page_size: Option<u32>,
}

/// Pagination data for paged responses.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "page": 1,
    "pageSize": 1,
    "total": 4352561,
}))]
pub struct PaginationData {
    /// Current page number. 1-indexed.
    #[schema(minimum = 1, example = 1)]
    pub page: u32,
    /// Number of items per page.
    #[schema(minimum = 1, example = 1)]
    #[schema(example = 1)]
    pub page_size: u32,
    /// Total number of items across all pages.
    #[schema(minimum = 0, example = 10467367)]
    pub total: u64,
}

/// Paged data response.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponse<T> {
    /// Pagination data.
    pub pagination: PaginationData,
    /// Data on the current page.
    pub data: Vec<T>,
}
