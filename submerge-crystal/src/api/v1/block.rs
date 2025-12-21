use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::{
    api::{get_page_number_and_size, ServiceState},
    persistence::{api::block::CrystalBlockAPIPostgreSQLStorage, CrystalPostgreSQLStorage},
    types::api::{
        dto::{
            pagination::PaginationData,
            request::block::{BlockQuery, BlockReference},
            response::{
                block::{BlockList, PaginatedBlockList},
                error::{BadRequest, InternalServerError, NotFound, TooManyRequests},
            },
        },
        error::APIError,
    },
};

#[utoipa::path(
    get,
    path = "/blocks",
    tag = "block",
    summary = "Get blocks",
    description = "Returns all blocks from the database that satisfy the query parameters. It will return a paginated response, ordered descending by block number.",
    params(BlockQuery),
    responses(
        (
            status = 200,
            response = PaginatedBlockList,
        ),
        (
            status = 400,
            response = BadRequest,
        ),
        (
            status = 429,
            response = TooManyRequests,
        ),
        (
            status = 500,
            response = InternalServerError,
        )
    )
)]
pub(crate) async fn get_blocks(
    State(state): State<ServiceState>,
    Query(query): Query<BlockQuery>,
) -> Result<Json<PaginatedBlockList>, APIError> {
    let (page, page_size) = get_page_number_and_size(query.page, query.page_size, false)?;
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
    let response = PaginatedBlockList {
        data,
        pagination: PaginationData {
            page,
            page_size,
            total: total_count,
        },
    };
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/blocks/{block_ref}",
    tag = "block",
    summary = "Get blocks by reference",
    description = "If a hash is passed, returns the matching block. If a number is passed, gives the blocks by that number - could be multiple blocks if there's a pruned block in that slot.",
    params(
        (
            "block_ref" = String,
            Path,
            description = "Block reference. Either a block number (integer ≥ 0), or a block hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
    ),
    responses(
        (
            status = 200,
            response = BlockList,
        ),
        (
            status = 400,
            response = BadRequest,
        ),
        (
            status = 404,
            response = NotFound,
        ),
        (
            status = 429,
            response = TooManyRequests,
        ),
        (
            status = 500,
            response = InternalServerError,
        )
    )
)]
pub(crate) async fn get_blocks_by_reference(
    State(state): State<ServiceState>,
    Path(block_reference): Path<String>,
) -> Result<Json<BlockList>, APIError> {
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
                Ok(Json(BlockList(data)))
            }
        }
        Ok(BlockReference::Hash(hash)) => match &state.postgres.get_block_by_hash(&hash).await {
            Ok(Some(row)) => Ok(Json(BlockList(vec![row.try_into()?]))),
            _ => Err(APIError::BlockNotFoundWithHash(hash)),
        },
        Err(message) => Err(APIError::BadRequest(message)),
    }
}
