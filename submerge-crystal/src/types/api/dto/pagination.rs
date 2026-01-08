use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub(crate) struct PaginationQuery {
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
    "total": 324,
}))]
pub(crate) struct PaginationData {
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

/// Pagination data for cursor responses.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "pageSize": 1,
    "nextCursor": "eyJjdXJzb3JfcG9zaXRpb24iOnsiYmxvY2tfbnVtYmVyIjozMzY0MzMzLCJibG9ja19oYXNoX2hleCI6IjB4ZjdlMjkyYWQ3ZDNkYzE4MzUzOWYwOGM4NDgwMmNiMDc2ZTc5NjNkYjA2NTA3MjAwNTY1M2NjNWU4YzdkMTE3MyIsImluZGV4IjowfSwicXVlcnkiOnsiaW5jbHVkZV9hcmdzIjpmYWxzZX19",
}))]
pub(crate) struct CursorPaginationData {
    /// Number of items per cursor page.
    #[schema(minimum = 1, example = 1)]
    #[schema(example = 1)]
    pub page_size: u32,
    /// Cursor for the next page, `null` if there's no next page.
    pub next_cursor: Option<String>,
}
