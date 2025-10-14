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
            extrinsic::{BlockExtrinsicQuery, ExtrinsicDTO, ExtrinsicQuery},
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
) -> Result<Json<PagedResponse<ExtrinsicDTO>>, APIError> {
    let page = query.pagination.get_page()?;
    let page_size = query
        .pagination
        .get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;
    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_extrinsic_count(&query),
        state.postgres.get_extrinsics(page, page_size, &query),
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

pub(crate) async fn get_extrinsics_by_block_reference(
    State(state): State<ServiceState>,
    Path(block_reference): Path<String>,
    Query(query): Query<BlockExtrinsicQuery>,
) -> Result<Json<PagedResponse<ExtrinsicDTO>>, APIError> {
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
                state
                    .postgres
                    .get_extrinsic_count_by_block_number(block_number, &query),
                state.postgres.get_extrinsics_by_block_number(
                    page,
                    page_size,
                    block_number,
                    &query
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
        Ok(BlockReference::Hash(block_hash)) => {
            if !state.postgres.block_exists_by_hash(&block_hash).await? {
                return Err(APIError::BlockNotFoundWithHash(block_hash));
            }
            let (total_count, rows) = tokio::try_join!(
                state
                    .postgres
                    .get_extrinsic_count_by_block_hash(&block_hash, &query),
                state
                    .postgres
                    .get_extrinsics_by_block_hash(page, page_size, &block_hash, &query),
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

pub(crate) async fn get_extrinsics_by_block_reference_and_index(
    State(state): State<ServiceState>,
    Path((block_reference, index)): Path<(String, u32)>,
) -> Result<Json<Vec<ExtrinsicDTO>>, APIError> {
    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(block_number)) => {
            if !state.postgres.block_exists_by_number(block_number).await? {
                return Err(APIError::BlockNotFoundWithNumber(block_number));
            }
            let rows = state
                .postgres
                .get_extrinsics_by_block_number_and_index(block_number, index)
                .await?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.try_into()?);
            }
            Ok(Json(data))
        }
        Ok(BlockReference::Hash(block_hash)) => {
            if !state.postgres.block_exists_by_hash(&block_hash).await? {
                return Err(APIError::BlockNotFoundWithHash(block_hash));
            }
            let response = if let Some(row) = &state
                .postgres
                .get_extrinsic_by_block_hash_and_index(&block_hash, index)
                .await?
            {
                vec![row.try_into()?]
            } else {
                vec![]
            };
            Ok(Json(response))
        }
        Err(message) => Err(APIError::BadRequest(message)),
    }
}

pub(crate) async fn get_extrinsic_by_hash(
    State(state): State<ServiceState>,
    Path(extrinsic_hash): Path<String>,
) -> Result<Json<ExtrinsicDTO>, APIError> {
    let extrinsic_hash = match hex::decode(extrinsic_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid extrinsic hash: {e}"))),
    };
    if let Some(row) = &state
        .postgres
        .get_extrinsic_by_hash(&extrinsic_hash)
        .await?
    {
        Ok(Json(row.try_into()?))
    } else {
        Err(APIError::ExtrinsicNotFoundWithHash(extrinsic_hash))
    }
}
