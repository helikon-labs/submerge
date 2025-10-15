use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::{
    api::ServiceState,
    persistence::{api::call::CrystalCallAPIPostgreSQLStorage, CrystalPostgreSQLStorage as _},
    types::api::{
        dto::{
            block::BlockReference,
            call::{BlockCallQuery, CallDTO, CallQuery},
            pagination::{PagedResponse, PaginationData},
        },
        error::APIError,
    },
};

const MAX_PAGE_SIZE: u64 = 100;
const DEFAULT_PAGE_SIZE: u64 = 50;

pub(crate) async fn get_calls(
    State(state): State<ServiceState>,
    Query(query): Query<CallQuery>,
) -> Result<Json<PagedResponse<CallDTO>>, APIError> {
    let page = query.pagination.get_page()?;
    let page_size = query
        .pagination
        .get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;

    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_call_count(
            query.min_block_number,
            query.max_block_number,
            query.min_block_timestamp,
            query.max_block_timestamp,
            query.min_spec_version,
            query.max_spec_version,
            &query.pallet_name,
            &query.pallet_call_name,
        ),
        state.postgres.get_calls(
            query.min_block_number,
            query.max_block_number,
            query.min_block_timestamp,
            query.max_block_timestamp,
            query.min_spec_version,
            query.max_spec_version,
            &query.pallet_name,
            &query.pallet_call_name,
            page,
            page_size,
        ),
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

pub(crate) async fn get_calls_by_block_reference(
    State(state): State<ServiceState>,
    Path(block_reference): Path<String>,
    Query(query): Query<BlockCallQuery>,
) -> Result<Json<PagedResponse<CallDTO>>, APIError> {
    let page = query.pagination.get_page()?;
    let page_size = query
        .pagination
        .get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;
    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(block_number)) => {
            if !state.postgres.block_exists_by_number(block_number).await? {
                return Err(APIError::BlockNotFoundWithNumber(block_number));
            }
            let (total_count, rows) = tokio::try_join!(
                state.postgres.get_call_count_by_block_number(
                    block_number,
                    &query.pallet_name,
                    &query.pallet_call_name,
                ),
                state.postgres.get_calls_by_block_number(
                    block_number,
                    &query.pallet_name,
                    &query.pallet_call_name,
                    page,
                    page_size,
                ),
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
        Ok(BlockReference::Hash(block_hash)) => {
            if !state.postgres.block_exists_by_hash(&block_hash).await? {
                return Err(APIError::BlockNotFoundWithHash(block_hash));
            }
            let (total_count, rows) = tokio::try_join!(
                state.postgres.get_call_count_by_block_hash(
                    &block_hash,
                    &query.pallet_name,
                    &query.pallet_call_name,
                ),
                state.postgres.get_calls_by_block_hash(
                    &block_hash,
                    &query.pallet_name,
                    &query.pallet_call_name,
                    page,
                    page_size,
                ),
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
        Err(message) => Err(APIError::BadRequest(message)),
    }
}
