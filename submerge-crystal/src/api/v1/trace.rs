use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse as _, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::StatusCode;

use crate::{
    api::{get_page_number_and_size, get_page_size, ServiceState},
    persistence::{
        api::{
            block::CrystalBlockAPIPostgreSQLStorage as _, trace::CrystalTraceAPIPostgreSQLStorage,
        },
        CrystalPostgreSQLStorage as _,
    },
    types::api::{
        dto::{
            pagination::{CursorPaginationData, PaginationData},
            request::{
                block::BlockReference,
                trace::{BlockTraceQuery, TraceQuery},
            },
            response::{
                error::{BadRequest, InternalServerError, NotFound, TooManyRequests},
                hex::HexString,
                trace::{
                    CursorTraceList, PaginatedTraceList, TraceCursorPayload, TraceCursorPosition,
                    TraceDTO,
                },
            },
        },
        error::APIError,
    },
};

#[utoipa::path(
    get,
    path = "/traces",
    tag = "trace",
    summary = "Get traces",
    description = "Returns all traces from the database that satisfy the query parameters. It will return a paginated response, ordered descending by block number, then ascending trace index.",
    params(TraceQuery),
    responses(
        (
            status = 200,
            response = CursorTraceList,
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
pub(crate) async fn get_traces(
    State(state): State<ServiceState>,
    Query(query): Query<TraceQuery>,
) -> Result<Json<CursorTraceList>, APIError> {
    query.validate_next_cursor_mutually_exclusive()?;
    let (cursor_position, query) = if let Some(cursor) = query.next_cursor {
        let decoded = URL_SAFE_NO_PAD.decode(cursor)?;
        let cursor_payload: TraceCursorPayload = serde_json::from_slice(&decoded)?;
        (Some(cursor_payload.cursor_position), cursor_payload.query)
    } else {
        (None, query)
    };
    let page_size = get_page_size(query.page_size, false)?;
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

    let key_prefix = if let Some(key_prefix) = query.key_prefix.as_ref() {
        Some(hex::decode(key_prefix.0.trim_start_matches("0x"))?)
    } else {
        None
    };
    let key_params = if let Some(key_params) = query.key_params.as_ref() {
        Some(hex::decode(key_params.0.trim_start_matches("0x"))?)
    } else {
        None
    };
    let rows = state
        .postgres
        .get_traces(
            cursor_position,
            min_block_number,
            max_block_number,
            key_prefix.as_deref(),
            key_params.as_deref(),
            page_size,
        )
        .await?;
    let mut data: Vec<TraceDTO> = Vec::new();
    for row in rows.iter() {
        data.push(row.try_into()?);
    }
    let next_cursor = if data.len() < page_size as usize {
        None
    } else if let Some(last_trace) = data.last() {
        let cursor_payload = TraceCursorPayload {
            cursor_position: TraceCursorPosition {
                block_number: last_trace.block_number,
                block_hash_hex: last_trace.block_hash.0.clone(),
                index: last_trace.index,
            },
            query,
        };
        let cursor = serde_json::to_string(&cursor_payload)?;
        let cursor_encoded = URL_SAFE_NO_PAD.encode(cursor.as_bytes());
        Some(cursor_encoded)
    } else {
        None
    };
    let response = CursorTraceList {
        pagination: CursorPaginationData {
            page_size,
            next_cursor,
        },
        data,
    };
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/blocks/{block_ref}/traces",
    tag = "trace",
    summary = "Get block traces",
    description = "If a hash is passed, returns the traces for the matching block. If a number is passed, returns the traces for the blocks with that number - could be multiple blocks if there's a pruned block in that slot.",
    params(
        (
            "block_ref" = String,
            Path,
            description = "Block reference for the traces. Either a block number (integer ≥ 0), or a block hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
        BlockTraceQuery,
    ),
    responses(
        (
            status = 200,
            response = PaginatedTraceList,
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
pub(crate) async fn get_traces_by_block_reference(
    State(state): State<ServiceState>,
    Path(block_reference): Path<String>,
    Query(query): Query<BlockTraceQuery>,
) -> Result<Json<PaginatedTraceList>, APIError> {
    let (page, page_size) = get_page_number_and_size(query.page, query.page_size, false)?;

    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(block_number)) => {
            if !state.postgres.block_exists_by_number(block_number).await? {
                return Err(APIError::BlockNotFoundWithNumber(block_number));
            }
            let (total_count, rows) = tokio::try_join!(
                state.postgres.get_trace_count_by_block_number(block_number),
                state
                    .postgres
                    .get_traces_by_block_number(block_number, page, page_size),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.try_into()?);
            }
            let response = PaginatedTraceList {
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
                state.postgres.get_trace_count_by_block_hash(&block_hash),
                state
                    .postgres
                    .get_traces_by_block_hash(&block_hash, page, page_size),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.try_into()?);
            }
            let response = PaginatedTraceList {
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
    path = "/traces/{trace_hash}",
    tag = "trace",
    summary = "Get trace by hash",
    description = "Returns the trace by its hash.",
    params(
        (
            "trace_hash" = String,
            Path,
            description = "Trace hash in hex (with or without `0x` prefix, case-insensitive).",
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
            description = "Trace with the given hash.",
            body = TraceDTO,
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
pub(crate) async fn get_trace_by_hash(
    State(state): State<ServiceState>,
    Path(trace_hash): Path<String>,
) -> Result<Json<TraceDTO>, APIError> {
    let trace_hash = match hex::decode(trace_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid trace hash: {e}"))),
    };
    if let Some(row) = &state.postgres.get_trace_by_hash(&trace_hash).await? {
        Ok(Json(row.try_into()?))
    } else {
        Err(APIError::TraceNotFoundWithHash(trace_hash))
    }
}

#[utoipa::path(
    get,
    path = "/traces/{trace_hash}/value",
    tag = "trace",
    summary = "Get trace value",
    description = "Returns the value of a trace record by its hash.",
    params(
        (
            "trace_hash" = String,
            Path,
            description = "Trace hash in hex (with or without `0x` prefix, case-insensitive).",
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
            description = "SCALE-encoded value of the storage trace.",
            body = HexString,
        ),
        (
            status = 204,
            description = "Trace exists but has no value (null).",
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
pub(crate) async fn get_trace_value_by_hash(
    State(state): State<ServiceState>,
    Path(trace_hash): Path<String>,
) -> Result<Response, APIError> {
    let trace_hash = match hex::decode(trace_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid trace hash: {e}"))),
    };
    if !state.postgres.trace_exists_by_hash(&trace_hash).await? {
        return Err(APIError::TraceNotFoundWithHash(trace_hash));
    }
    if let Some(value_bytes) = state.postgres.get_trace_value_by_hash(&trace_hash).await? {
        let hex_string = format!("0x{}", hex::encode(value_bytes));
        Ok(([(header::CONTENT_TYPE, "text/plain")], hex_string).into_response())
    } else {
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}
