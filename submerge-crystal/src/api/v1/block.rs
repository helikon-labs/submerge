use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::{
    api::ServiceState,
    persistence::{api::block::CrystalBlockAPIPostgreSQLStorage, CrystalPostgreSQLStorage},
    types::api::{
        dto::{
            block::{BlockDTO, BlockQuery, BlockReference},
            pagination::{PagedResponse, PaginationData},
        },
        error::APIError,
    },
};

const DEFAULT_PAGE_SIZE: u64 = 50;
const MAX_PAGE_SIZE: u64 = 100;

pub(crate) async fn get_blocks(
    State(state): State<ServiceState>,
    Query(query): Query<BlockQuery>,
) -> Result<Json<PagedResponse<BlockDTO>>, APIError> {
    let page = query.pagination.get_page()?;
    let page_size = query
        .pagination
        .get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;
    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_block_count(&query),
        state.postgres.get_block_rows(page, page_size, &query),
    )?;
    let response = PagedResponse {
        pagination: PaginationData {
            page,
            page_size,
            total: total_count,
        },
        data: rows.iter().map(|row| row.into()).collect(),
    };
    Ok(Json(response))
}

pub(crate) async fn get_blocks_by_reference(
    State(state): State<ServiceState>,
    Path(block_reference): Path<String>,
) -> Result<Json<Vec<BlockDTO>>, APIError> {
    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(number)) => {
            let rows = state.postgres.get_blocks_by_number(number).await?;
            if rows.is_empty() {
                Err(APIError::BlockNotFoundWithNumber(number))
            } else {
                Ok(Json(rows.iter().map(|row| row.into()).collect()))
            }
        }
        Ok(BlockReference::Hash(hash)) => match &state.postgres.get_block_by_hash(&hash).await {
            Ok(Some(row)) => Ok(Json(vec![row.into()])),
            _ => Err(APIError::BlockNotFoundWithHash(hash)),
        },
        Err(message) => Err(APIError::BadRequest(message)),
    }
}
