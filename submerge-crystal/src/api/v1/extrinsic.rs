use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::{
    api::{get_page_number_and_size, ServiceState},
    persistence::{
        api::{
            block::CrystalBlockAPIPostgreSQLStorage, call::CrystalCallAPIPostgreSQLStorage as _,
            extrinsic::CrystalExtrinsicAPIPostgreSQLStorage,
        },
        CrystalPostgreSQLStorage,
    },
    types::api::{
        dto::{
            pagination::PaginationData,
            request::{
                block::BlockReference,
                extrinsic::{BlockExtrinsicQuery, ExtrinsicQuery},
            },
            response::{
                call::CallDTO,
                error::{BadRequest, InternalServerError, NotFound, TooManyRequests},
                extrinsic::{ExtrinsicDTO, ExtrinsicList, PaginatedExtrinsicList},
            },
        },
        error::APIError,
    },
};

#[utoipa::path(
    get,
    path = "/extrinsics",
    tag = "extrinsic",
    summary = "Get extrinsics",
    description = "Returns all extrinsics from the database that satisfy the query parameters. It will return a paginated response, ordered descending by block number and ascending extrinsic index.",
    params(ExtrinsicQuery),
    responses(
        (
            status = 200,
            response = PaginatedExtrinsicList,
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
pub(crate) async fn get_extrinsics(
    State(state): State<ServiceState>,
    Query(query): Query<ExtrinsicQuery>,
) -> Result<Json<PaginatedExtrinsicList>, APIError> {
    let (page, page_size) = get_page_number_and_size(query.page, query.page_size)?;
    let Ok(signer_multi_address) = query.get_signer_multi_address() else {
        return Err(APIError::InvalidExtrinsicSigner(
            query.signer.unwrap_or("".to_string()),
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
        state.postgres.get_extrinsic_count(
            min_block_number,
            max_block_number,
            query.is_signed,
            &signer_multi_address,
        ),
        state.postgres.get_extrinsics(
            min_block_number,
            max_block_number,
            query.is_signed,
            &signer_multi_address,
            page,
            page_size,
        ),
    )?;
    let mut data = Vec::new();
    for row in rows.iter() {
        data.push(row.try_into()?);
    }
    let response = PaginatedExtrinsicList {
        pagination: PaginationData {
            page,
            page_size,
            total: total_count,
        },
        data,
    };
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/blocks/{block_ref}/extrinsics",
    tag = "call",
    summary = "Get block extrinsics",
    description = "If a hash is passed, returns the extrinsics for the matching block. If a number is passed, gives the extrinsis for the block by that number - could be multiple blocks if there's a pruned block in that slot.",
    params(
        (
            "block_ref" = String,
            Path,
            description = "Block reference for the extrinsics. Either a block number (integer ≥ 0), or a block hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
            example = "13453264",
        ),
        BlockExtrinsicQuery,
    ),
    responses(
        (
            status = 200,
            response = PaginatedExtrinsicList,
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
pub(crate) async fn get_extrinsics_by_block_reference(
    State(state): State<ServiceState>,
    Path(block_reference): Path<String>,
    Query(query): Query<BlockExtrinsicQuery>,
) -> Result<Json<PaginatedExtrinsicList>, APIError> {
    let (page, page_size) = get_page_number_and_size(query.page, query.page_size)?;
    let Ok(signer_multi_address) = query.get_signer_multi_address() else {
        return Err(APIError::InvalidExtrinsicSigner(
            query.signer.unwrap_or("".to_string()),
        ));
    };
    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(block_number)) => {
            if !state.postgres.block_exists_by_number(block_number).await? {
                return Err(APIError::BlockNotFoundWithNumber(block_number));
            }
            let (total_count, rows) = tokio::try_join!(
                state.postgres.get_extrinsic_count_by_block_number(
                    block_number,
                    query.is_signed,
                    &signer_multi_address,
                ),
                state.postgres.get_extrinsics_by_block_number(
                    block_number,
                    query.is_signed,
                    &signer_multi_address,
                    page,
                    page_size,
                ),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.try_into()?);
            }
            let response = PaginatedExtrinsicList {
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
                state.postgres.get_extrinsic_count_by_block_hash(
                    &block_hash,
                    query.is_signed,
                    &signer_multi_address,
                ),
                state.postgres.get_extrinsics_by_block_hash(
                    &block_hash,
                    query.is_signed,
                    &signer_multi_address,
                    page,
                    page_size,
                ),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.try_into()?);
            }
            let response = PaginatedExtrinsicList {
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

#[utoipa::path(
    get,
    path = "/blocks/{block_ref}/extrinsics/{extrinsic_index}",
    tag = "block",
    summary = "Get block extrinsics by index",
    description = "Returns the extrinsics in the given block at the given index. It can return multiple extrinsics if a number is passed and there's a pruned block in that slot.",
    params(
        (
            "block_ref" = String,
            Path,
            description = "Block reference for the extrinsic. Either a block number (integer ≥ 0), or a block hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
        (
            "extrinsic_index" = u32,
            Path,
            description = "0-based index for the extrinsic in the block.",
        ),
    ),
    responses(
        (
            status = 200,
            response = ExtrinsicList,
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
pub(crate) async fn get_extrinsics_by_block_reference_and_index(
    State(state): State<ServiceState>,
    Path((block_reference, index)): Path<(String, u32)>,
) -> Result<Json<ExtrinsicList>, APIError> {
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
            Ok(Json(ExtrinsicList(data)))
        }
        Ok(BlockReference::Hash(block_hash)) => {
            if !state.postgres.block_exists_by_hash(&block_hash).await? {
                return Err(APIError::BlockNotFoundWithHash(block_hash));
            }
            let data = if let Some(row) = &state
                .postgres
                .get_extrinsic_by_block_hash_and_index(&block_hash, index)
                .await?
            {
                vec![row.try_into()?]
            } else {
                vec![]
            };
            Ok(Json(ExtrinsicList(data)))
        }
        Err(message) => Err(APIError::BadRequest(message)),
    }
}

#[utoipa::path(
    get,
    path = "/extrinsics/{extrinsic_hash}",
    tag = "extrinsic",
    summary = "Get extrinsic by hash",
    description = "Returns the extrinsic by its hash.",
    params(
        (
            "extrinsic_hash" = String,
            Path,
            description = "Extrinsic hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
    ),
    responses(
        (
            status = 200,
            headers(
                ("X-RateLimit-Limit" = u32),
                ("X-RateLimit-Remaining" = u32),
            ),
            description = "Extrinsic with the given hash.",
            body = ExtrinsicDTO,
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

#[utoipa::path(
    get,
    path = "/extrinsics/{extrinsic_hash}/call",
    tag = "extrinsic",
    summary = "Get extrinsic root call",
    description = "Returns the root call of an extrinsic by its hash.",
    params(
        (
            "extrinsic_hash" = String,
            Path,
            description = "Extrinsic hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
    ),
    responses(
        (
            status = 200,
            headers(
                ("X-RateLimit-Limit" = u32),
                ("X-RateLimit-Remaining" = u32),
            ),
            description = "The root call of the extrinsic.",
            body = CallDTO,
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
pub(crate) async fn get_extrinsic_root_call_by_hash(
    State(state): State<ServiceState>,
    Path(extrinsic_hash): Path<String>,
) -> Result<Json<CallDTO>, APIError> {
    let extrinsic_hash = match hex::decode(extrinsic_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid extrinsic hash: {e}"))),
    };
    if let Some(row) = &state
        .postgres
        .get_extrinsic_root_call_by_hash(&extrinsic_hash)
        .await?
    {
        Ok(Json(row.into()))
    } else {
        Err(APIError::ExtrinsicNotFoundWithHash(extrinsic_hash))
    }
}
