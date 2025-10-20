use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    api::ServiceState,
    persistence::api::trace::CrystalTraceAPIPostgreSQLStorage,
    types::api::{
        dto::{
            pagination::{PagedResponse, PaginationData},
            // block::BlockReference,
            trace::{TraceDTO, TraceQuery},
        },
        error::APIError,
    },
};

const MAX_PAGE_SIZE: u64 = 25;
const DEFAULT_PAGE_SIZE: u64 = 10;

pub(crate) async fn get_traces(
    State(state): State<ServiceState>,
    Query(query): Query<TraceQuery>,
) -> Result<Json<PagedResponse<TraceDTO>>, APIError> {
    let page = query.pagination.get_page()?;
    let page_size = query
        .pagination
        .get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;
    let key = if let Some(key) = query.key.as_deref() {
        Some(hex::decode(key.trim_start_matches("0x"))?)
    } else {
        None
    };
    let key_prefix = if let Some(key_prefix) = query.key_prefix.as_deref() {
        Some(hex::decode(key_prefix.trim_start_matches("0x"))?)
    } else {
        None
    };
    let (total_count, rows) = tokio::try_join!(
        state
            .postgres
            .get_trace_count(key.as_deref(), key_prefix.as_deref(),),
        state
            .postgres
            .get_traces(key.as_deref(), key_prefix.as_deref(), page, page_size,),
    )?;
    let mut data = Vec::new();
    for row in rows.iter() {
        data.push(row.into());
    }
    let response = PagedResponse {
        pagination: PaginationData {
            page,
            page_size,
            total: total_count,
        },
        data,
    };
    Ok(Json(response))
}
