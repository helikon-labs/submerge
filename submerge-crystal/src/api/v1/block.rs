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

pub(crate) async fn get_blocks(
    State(state): State<ServiceState>,
    Query(query): Query<BlockQuery>,
) -> Result<Json<PagedResponse<BlockDTO>>, APIError> {
    let page = query.pagination.get_page()?;
    let page_size = query
        .pagination
        .get_page_size(super::DEFAULT_PAGE_SIZE, super::MAX_PAGE_SIZE)?;
    let Ok(author_multi_address) = query.get_author_multi_address() else {
        return Err(APIError::InvalidBlockAuthor(
            query.author.unwrap_or("".to_string()),
        ));
    };
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

    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_block_count(
            query.status,
            min_block_number,
            max_block_number,
            &author_multi_address,
        ),
        state.postgres.get_block_rows(
            query.status,
            min_block_number,
            max_block_number,
            &author_multi_address,
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
                let mut data = Vec::new();
                for row in rows.iter() {
                    data.push(row.try_into()?);
                }
                Ok(Json(data))
            }
        }
        Ok(BlockReference::Hash(hash)) => match &state.postgres.get_block_by_hash(&hash).await {
            Ok(Some(row)) => Ok(Json(vec![row.try_into()?])),
            _ => Err(APIError::BlockNotFoundWithHash(hash)),
        },
        Err(message) => Err(APIError::BadRequest(message)),
    }
}
