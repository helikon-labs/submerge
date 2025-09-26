use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::{
    api::ServiceState,
    persistence::{api::extrinsic::CrystalExtrinsicAPIPostgreSQLStorage, CrystalPostgreSQLStorage},
    types::api::{
        dto::{
            block::BlockReference,
            extrinsic::{BlockExtrinsicQuery, Extrinsic, ExtrinsicQuery},
            pagination::{PagedResponse, PaginationData},
        },
        error::APIError,
    },
};

const MAX_PAGE_SIZE: u64 = 100;
const DEFAULT_PAGE_SIZE: u64 = 50;

pub(crate) async fn get_extrinsics(
    State(state): State<ServiceState>,
    Query(query): Query<ExtrinsicQuery>,
) -> Result<Json<PagedResponse<Extrinsic>>, APIError> {
    let page_number = query.pagination.get_page_number()?;
    let page_size = query
        .pagination
        .get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;
    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_extrinsic_count(&query),
        state
            .postgres
            .get_extrinsic_rows(page_number, page_size, &query),
    )?;
    let response = PagedResponse {
        pagination: PaginationData {
            page_number,
            page_size,
            total_count,
        },
        data: rows.iter().map(|row| row.into()).collect(),
    };
    Ok(Json(response))
}

pub(crate) async fn get_extrinsics_by_block_reference(
    State(state): State<ServiceState>,
    Path(block_reference): Path<String>,
    Query(query): Query<BlockExtrinsicQuery>,
) -> Result<Json<PagedResponse<Extrinsic>>, APIError> {
    let page_number = query.pagination.get_page_number()?;
    let page_size = query
        .pagination
        .get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;
    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(block_number)) => {
            if !state.postgres.block_exists_by_number(block_number).await? {
                return Err(APIError::BlockNotFoundWithNumber(block_number));
            }
            let (total_count, rows) = tokio::try_join!(
                state
                    .postgres
                    .get_extrinsic_row_count_by_block_number(block_number, &query),
                state.postgres.get_extrinsic_rows_by_block_number(
                    page_number,
                    page_size,
                    block_number,
                    &query
                ),
            )?;
            let response = PagedResponse {
                pagination: PaginationData {
                    page_number,
                    page_size,
                    total_count,
                },
                data: rows.iter().map(|row| row.into()).collect(),
            };
            Ok(Json(response))
        }
        Ok(BlockReference::Hash(block_hash)) => {
            if !state.postgres.block_exists_by_hash(&block_hash).await? {
                return Err(APIError::BlockNotFoundWithHash(block_hash));
            }
            let (total_count, rows) = tokio::try_join!(
                state
                    .postgres
                    .get_extrinsic_row_count_by_block_hash(&block_hash, &query),
                state.postgres.get_extrinsic_rows_by_block_hash(
                    page_number,
                    page_size,
                    &block_hash,
                    &query
                ),
            )?;
            let response = PagedResponse {
                pagination: PaginationData {
                    page_number,
                    page_size,
                    total_count,
                },
                data: rows.iter().map(|row| row.into()).collect(),
            };
            Ok(Json(response))
        }
        Err(message) => Err(APIError::BadRequest(message)),
    }
}
