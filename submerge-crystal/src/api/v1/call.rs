use axum::{
    extract::{Path, Query, State},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use crate::{
    api::{get_page_number_and_size, get_page_size, ServiceState},
    persistence::{
        api::{
            block::CrystalBlockAPIPostgreSQLStorage as _, call::CrystalCallAPIPostgreSQLStorage,
            extrinsic::CrystalExtrinsicAPIPostgreSQLStorage,
            metadata::CrystalMetadataAPIPostgreSQLStorage,
        },
        CrystalPostgreSQLStorage as _,
    },
    types::api::{
        dto::{
            pagination::{CursorPaginationData, PaginationData},
            request::{
                block::BlockReference,
                call::{BlockCallQuery, CallQuery, IncludeCallArgsParam},
            },
            response::{
                call::{
                    CallArgs, CallCursorPayload, CallCursorPosition, CallDTO, CursorCallList,
                    PaginatedCallList,
                },
                error::{BadRequest, InternalServerError, NotFound, TooManyRequests},
                extrinsic::ExtrinsicDTO,
            },
        },
        error::APIError,
    },
};

#[utoipa::path(
    get,
    path = "/calls",
    tag = "call",
    summary = "Get calls",
    description = "Returns all calls from the database that satisfy the query parameters. It will return a paginated response, ordered descending by block number, then call id.",
    params(CallQuery),
    responses(
        (
            status = 200,
            response = CursorCallList,
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
pub(crate) async fn get_calls(
    State(state): State<ServiceState>,
    Query(query): Query<CallQuery>,
) -> Result<Json<CursorCallList>, APIError> {
    query.validate_next_cursor_mutually_exclusive()?;
    let (cursor_position, query) = if let Some(cursor) = query.next_cursor {
        // TODO validate that no other query params are set
        let decoded = URL_SAFE_NO_PAD.decode(cursor)?;
        let cursor_payload: CallCursorPayload = serde_json::from_slice(&decoded)?;
        (Some(cursor_payload.cursor_position), cursor_payload.query)
    } else {
        (None, query)
    };
    let page_size = get_page_size(query.page_size, query.include_args)?;
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
    let metadata_call_ids = if let Some(call_name) = query.call_name.as_deref() {
        let call_name = call_name.trim();
        if call_name.is_empty() {
            None
        } else {
            let metadata_call_ids = state
                .postgres
                .get_metadata_call_ids_by_pallet_name_and_call_name(
                    query.min_spec_version,
                    query.max_spec_version,
                    query.pallet_name.as_deref(),
                    call_name,
                )
                .await?;
            Some(metadata_call_ids)
        }
    } else if query.pallet_name.is_some() {
        return Err(APIError::BadRequest(
            "call_name should not be empty when pallet_name is set.".to_string(),
        ));
    } else {
        None
    };

    let rows = state
        .postgres
        .get_calls(
            cursor_position,
            min_block_number,
            max_block_number,
            metadata_call_ids,
            page_size,
            query.is_signed,
            query.include_args,
        )
        .await?;
    let mut data: Vec<CallDTO> = Vec::new();
    for row in rows.iter() {
        data.push(row.into());
    }
    let next_cursor = if data.len() < page_size as usize {
        None
    } else if let Some(last_call) = data.last() {
        let cursor_payload = CallCursorPayload {
            cursor_position: CallCursorPosition {
                block_number: last_call.block_number,
                block_hash_hex: last_call.block_hash.0.clone(),
                call_index: last_call.call_index.clone(),
            },
            query,
        };
        let cursor = serde_json::to_string(&cursor_payload)?;
        let cursor_encoded = URL_SAFE_NO_PAD.encode(cursor.as_bytes());
        Some(cursor_encoded)
    } else {
        None
    };
    let response = CursorCallList {
        data,
        pagination: CursorPaginationData {
            page_size,
            next_cursor,
        },
    };
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/blocks/{block_ref}/calls",
    tag = "call",
    summary = "Get block calls",
    description = "If a hash is passed, returns the calls for the matching block. If a number is passed, gives the calls for the blocks with that number - could be multiple blocks if there's a pruned block in that slot.",
    params(
        (
            "block_ref" = String,
            Path,
            description = "Block reference for the calls. Either a block number (integer ≥ 0), or a block hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
        BlockCallQuery,
    ),
    responses(
        (
            status = 200,
            response = PaginatedCallList,
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
pub(crate) async fn get_calls_by_block_reference(
    State(state): State<ServiceState>,
    Path(block_reference): Path<String>,
    Query(query): Query<BlockCallQuery>,
) -> Result<Json<PaginatedCallList>, APIError> {
    let (page, page_size) =
        get_page_number_and_size(query.page, query.page_size, query.include_args)?;
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
                    query.is_signed,
                ),
                state.postgres.get_calls_by_block_number(
                    block_number,
                    &query.pallet_name,
                    &query.pallet_call_name,
                    page,
                    page_size,
                    query.is_signed,
                    query.include_args,
                ),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.into());
            }
            let response = PaginatedCallList {
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
                    query.is_signed,
                ),
                state.postgres.get_calls_by_block_hash(
                    &block_hash,
                    &query.pallet_name,
                    &query.pallet_call_name,
                    page,
                    page_size,
                    query.is_signed,
                    query.include_args,
                ),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.into());
            }
            let response = PaginatedCallList {
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
    path = "/blocks/{block_ref}/extrinsics/{extrinsic_index}/calls",
    tag = "call",
    summary = "Get block extrinsic calls",
    description = "Returns the calls for extrinsic in a block by block reference and 0-based extrinsic index.",
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
        BlockCallQuery,
    ),
    responses(
        (
            status = 200,
            response = PaginatedCallList,
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
pub(crate) async fn get_calls_by_block_reference_and_extrinsic_index(
    State(state): State<ServiceState>,
    Path((block_reference, extrinsic_index)): Path<(String, u32)>,
    Query(query): Query<BlockCallQuery>,
) -> Result<Json<PaginatedCallList>, APIError> {
    let (page, page_size) =
        get_page_number_and_size(query.page, query.page_size, query.include_args)?;
    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(block_number)) => {
            if !state.postgres.block_exists_by_number(block_number).await? {
                return Err(APIError::BlockNotFoundWithNumber(block_number));
            }
            if !state
                .postgres
                .block_extrinsic_exists_by_number_and_index(block_number, extrinsic_index)
                .await?
            {
                return Err(APIError::BlockExtrinsicNotFoundWithNumberAndIndex(
                    block_number,
                    extrinsic_index,
                ));
            }
            let (total_count, rows) = tokio::try_join!(
                state
                    .postgres
                    .get_call_count_by_block_number_and_extrinsic_index(
                        block_number,
                        extrinsic_index,
                        &query.pallet_name,
                        &query.pallet_call_name,
                    ),
                state
                    .postgres
                    .get_calls_by_block_number_and_extrinsic_index(
                        block_number,
                        extrinsic_index,
                        &query.pallet_name,
                        &query.pallet_call_name,
                        page,
                        page_size,
                        query.include_args,
                    ),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.into());
            }
            let response = PaginatedCallList {
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
            if !state
                .postgres
                .block_extrinsic_exists_by_hash_and_index(&block_hash, extrinsic_index)
                .await?
            {
                return Err(APIError::BlockExtrinsicNotFoundWithHashAndIndex(
                    block_hash,
                    extrinsic_index,
                ));
            }
            let (total_count, rows) = tokio::try_join!(
                state
                    .postgres
                    .get_call_count_by_block_hash_and_extrinsic_index(
                        &block_hash,
                        extrinsic_index,
                        &query.pallet_name,
                        &query.pallet_call_name,
                    ),
                state.postgres.get_calls_by_block_hash_and_extrinsic_index(
                    &block_hash,
                    extrinsic_index,
                    &query.pallet_name,
                    &query.pallet_call_name,
                    page,
                    page_size,
                    query.include_args,
                ),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.into());
            }
            let response = PaginatedCallList {
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
    path = "/extrinsics/{extrinsic_hash}/calls",
    tag = "call",
    summary = "Get extrinsic calls",
    description = "Returns the calls in an extrinsic by extrinsic hash.",
    params(
        (
            "extrinsic_hash" = String,
            Path,
            description = "Extrinsic hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:0x)?[0-9a-fA-F]{64}$",
        ),
        BlockCallQuery,
    ),
    responses(
        (
            status = 200,
            response = PaginatedCallList,
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
pub(crate) async fn get_calls_by_extrinsic_hash(
    State(state): State<ServiceState>,
    Path(extrinsic_hash): Path<String>,
    Query(query): Query<BlockCallQuery>,
) -> Result<Json<PaginatedCallList>, APIError> {
    let extrinsic_hash = if let Ok(extrinsic_hash) =
        hex::decode(extrinsic_hash.trim_start_matches("0x"))
    {
        extrinsic_hash
    } else {
        return Err(APIError::BadRequest("Invalid extrinsic hash. It should be a hexadecimal string (with or without 0x prefix, case-insensitive).".to_string()));
    };
    if !state
        .postgres
        .extrinsic_exists_by_hash(&extrinsic_hash)
        .await?
    {
        return Err(APIError::ExtrinsicNotFoundWithHash(extrinsic_hash));
    }
    let (page, page_size) =
        get_page_number_and_size(query.page, query.page_size, query.include_args)?;
    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_call_count_by_extrinsic_hash(
            &extrinsic_hash,
            &query.pallet_name,
            &query.pallet_call_name,
        ),
        state.postgres.get_calls_by_extrinsic_hash(
            &extrinsic_hash,
            &query.pallet_name,
            &query.pallet_call_name,
            page,
            page_size,
            query.include_args,
        ),
    )?;
    let mut data = Vec::new();
    for row in rows.iter() {
        data.push(row.into());
    }
    let response = PaginatedCallList {
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
    path = "/calls/{call_hash}",
    tag = "call",
    summary = "Get call by hash",
    description = "Returns the call by its hash.",
    params(
        (
            "call_hash" = String,
            Path,
            description = "Call hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
        IncludeCallArgsParam,
    ),
    responses(
        (
            status = 200,
            headers(
                ("X-RateLimit-Limit" = u32),
                ("X-RateLimit-Remaining" = u32),
            ),
            description = "Call with the given hash.",
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
pub(crate) async fn get_call_by_hash(
    State(state): State<ServiceState>,
    Path(call_hash): Path<String>,
    Query(query): Query<IncludeCallArgsParam>,
) -> Result<Json<CallDTO>, APIError> {
    let call_hash = match hex::decode(call_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid call hash: {e}"))),
    };
    if let Some(row) = &state
        .postgres
        .get_call_by_hash(&call_hash, query.include_args)
        .await?
    {
        Ok(Json(row.into()))
    } else {
        Err(APIError::CallNotFoundWithHash(call_hash))
    }
}

#[utoipa::path(
    get,
    path = "/calls/{call_hash}/args",
    tag = "call",
    summary = "Get call arguments",
    description = "Returns the arguments of a runtime call by its hash.",
    params(
        (
            "call_hash" = String,
            Path,
            description = "Call hash in hex (with or without `0x` prefix, case-insensitive).",
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
            description = "Arguments for the runtime call with the given hash.",
            body = CallArgs,
            example = json!({
                "hash": "0xb778a81c1fd06d98b5ba1b37bb274101f7905ad5eca960f56ededf26248c4011",
                "args": {
                    "dest": {
                        "type": "Id",
                        "value": "0xc35b9a45aadc8bb998ba7c4d17bda4d7d8e31f90a754a65709d3a3a71ff8fa7a"
                    },
                    "value": "117284000000"
                }
            }),
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
pub(crate) async fn get_call_args_by_hash(
    State(state): State<ServiceState>,
    Path(call_hash): Path<String>,
) -> Result<Json<CallArgs>, APIError> {
    let call_hash = match hex::decode(call_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid call hash: {e}"))),
    };
    if let Some(args) = state.postgres.get_call_args_by_hash(&call_hash).await? {
        Ok(Json(CallArgs(args)))
    } else {
        Err(APIError::CallNotFoundWithHash(call_hash))
    }
}

#[utoipa::path(
    get,
    path = "/calls/{call_hash}/parent",
    tag = "call",
    summary = "Get parent call",
    description = "Returns a parent call by the sub call's hash.",
    params(
        (
            "call_hash" = String,
            Path,
            description = "Sub call hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
        IncludeCallArgsParam,
    ),
    responses(
        (
            status = 200,
            headers(
                ("X-RateLimit-Limit" = u32),
                ("X-RateLimit-Remaining" = u32),
            ),
            description = "Parent call for the given sub call hash.",
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
pub(crate) async fn get_parent_call_by_hash(
    State(state): State<ServiceState>,
    Path(call_hash): Path<String>,
    Query(query): Query<IncludeCallArgsParam>,
) -> Result<Json<CallDTO>, APIError> {
    let call_hash = match hex::decode(call_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid call hash: {e}"))),
    };
    if !state.postgres.call_exists_by_hash(&call_hash).await? {
        return Err(APIError::CallNotFoundWithHash(call_hash));
    }
    if let Some(row) = &state
        .postgres
        .get_parent_call_by_hash(&call_hash, query.include_args)
        .await?
    {
        Ok(Json(row.into()))
    } else {
        Err(APIError::ParentCallNotFoundForCallWithHash(call_hash))
    }
}

#[utoipa::path(
    get,
    path = "/calls/{call_hash}/subs",
    tag = "call",
    summary = "Get sub calls",
    description = "Returns sub calls call by a parent call's hash.",
    params(
        (
            "call_hash" = String,
            Path,
            description = "Parent call hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
        BlockCallQuery,
    ),
    responses(
        (
            status = 200,
            response = PaginatedCallList,
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
pub(crate) async fn get_sub_calls_by_hash(
    State(state): State<ServiceState>,
    Path(call_hash): Path<String>,
    Query(query): Query<BlockCallQuery>,
) -> Result<Json<PaginatedCallList>, APIError> {
    let call_hash = if let Ok(call_hash) = hex::decode(call_hash.trim_start_matches("0x")) {
        call_hash
    } else {
        return Err(APIError::BadRequest("Invalid call hash. It should be a hexadecimal string (with or without 0x prefix, case-insensitive).".to_string()));
    };
    if !state.postgres.call_exists_by_hash(&call_hash).await? {
        return Err(APIError::CallNotFoundWithHash(call_hash));
    }
    let (page, page_size) =
        get_page_number_and_size(query.page, query.page_size, query.include_args)?;
    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_sub_call_count_by_hash(
            &call_hash,
            &query.pallet_name,
            &query.pallet_call_name,
        ),
        state.postgres.get_sub_calls_by_hash(
            &call_hash,
            &query.pallet_name,
            &query.pallet_call_name,
            page,
            page_size,
            query.include_args,
        ),
    )?;
    let mut data = Vec::new();
    for row in rows.iter() {
        data.push(row.into());
    }
    let response: PaginatedCallList = PaginatedCallList {
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
    path = "/calls/{call_hash}/extrinsic",
    tag = "call",
    summary = "Get call extrinsic",
    description = "Returns the extrinsic of a call by call hash.",
    params(
        (
            "call_hash" = String,
            Path,
            description = "Hash of the extrinsic call in hex (with or without `0x` prefix, case-insensitive).",
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
            description = "The extrinsic that contains the call.",
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
pub(crate) async fn get_call_extrinsic_by_hash(
    State(state): State<ServiceState>,
    Path(call_hash): Path<String>,
) -> Result<Json<ExtrinsicDTO>, APIError> {
    let call_hash = match hex::decode(call_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid call hash: {e}"))),
    };
    if let Some(row) = &state
        .postgres
        .get_call_extrinsic_by_hash(&call_hash)
        .await?
    {
        Ok(Json(row.try_into()?))
    } else {
        Err(APIError::CallNotFoundWithHash(call_hash))
    }
}

#[utoipa::path(
    get,
    path = "/extrinsics/{extrinsic_hash}/call",
    tag = "call",
    summary = "Get extrinsic root call",
    description = "Returns the root call of an extrinsic by its hash.",
    params(
        (
            "extrinsic_hash" = String,
            Path,
            description = "Extrinsic hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
        IncludeCallArgsParam,
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
    Query(query): Query<IncludeCallArgsParam>,
) -> Result<Json<CallDTO>, APIError> {
    let extrinsic_hash = match hex::decode(extrinsic_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid extrinsic hash: {e}"))),
    };
    if let Some(row) = &state
        .postgres
        .get_extrinsic_root_call_by_hash(&extrinsic_hash, query.include_args)
        .await?
    {
        Ok(Json(row.into()))
    } else {
        Err(APIError::ExtrinsicNotFoundWithHash(extrinsic_hash))
    }
}
