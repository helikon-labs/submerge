use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::{
    api::{get_page_number_and_size, ServiceState},
    persistence::{
        api::{
            block::CrystalBlockAPIPostgreSQLStorage as _, trace::CrystalTraceAPIPostgreSQLStorage,
        },
        CrystalPostgreSQLStorage as _,
    },
    types::api::{
        dto::{
            block::BlockReference,
            pagination::{PagedResponse, PaginationData},
            trace::{BlockTraceQuery, TraceDTO, TraceQuery},
        },
        error::APIError,
    },
};

pub(crate) async fn get_traces(
    State(state): State<ServiceState>,
    Query(query): Query<TraceQuery>,
) -> Result<Json<PagedResponse<TraceDTO>>, APIError> {
    let (page, page_size) =
        get_page_number_and_size(query.pagination.page, query.pagination.page_size)?;
    let (min_block_number, max_block_number) = state
        .postgres
        .get_block_number_range(
            query.min_block_number,
            query.max_block_number,
            query.min_block_timestamp,
            query.max_block_timestamp,
            query.min_spec_version,
            query.max_spec_version,
        )
        .await?;
    let key_prefix = if let Some(key_prefix) = query.key_prefix.as_deref() {
        Some(hex::decode(key_prefix.trim_start_matches("0x"))?)
    } else {
        None
    };
    let key_params = if let Some(key_params) = query.key_params.as_deref() {
        Some(hex::decode(key_params.trim_start_matches("0x"))?)
    } else {
        None
    };
    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_trace_count(
            min_block_number,
            max_block_number,
            key_prefix.as_deref(),
            key_params.as_deref(),
        ),
        state.postgres.get_traces(
            min_block_number,
            max_block_number,
            key_prefix.as_deref(),
            key_params.as_deref(),
            page,
            page_size,
        ),
    )?;
    let mut data = Vec::new();
    for row in rows.iter() {
        data.push(row.try_into()?);
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

pub(crate) async fn get_traces_by_block_reference(
    State(state): State<ServiceState>,
    Path(block_reference): Path<String>,
    Query(query): Query<BlockTraceQuery>,
) -> Result<Json<PagedResponse<TraceDTO>>, APIError> {
    let (page, page_size) =
        get_page_number_and_size(query.pagination.page, query.pagination.page_size)?;
    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(block_number)) => {
            if !state.postgres.block_exists_by_number(block_number).await? {
                return Err(APIError::BlockNotFoundWithNumber(block_number));
            }
            let (total_count, rows) = tokio::try_join!(
                state
                    .postgres
                    .get_trace_count_by_block_number(block_number,),
                state
                    .postgres
                    .get_traces_by_block_number(block_number, page, page_size,),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.try_into()?);
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
        Ok(BlockReference::Hash(block_hash)) => {
            if !state.postgres.block_exists_by_hash(&block_hash).await? {
                return Err(APIError::BlockNotFoundWithHash(block_hash));
            }
            let (total_count, rows) = tokio::try_join!(
                state.postgres.get_trace_count_by_block_hash(&block_hash,),
                state
                    .postgres
                    .get_traces_by_block_hash(&block_hash, page, page_size,),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.try_into()?);
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
        Err(message) => Err(APIError::BadRequest(message)),
    }
}
