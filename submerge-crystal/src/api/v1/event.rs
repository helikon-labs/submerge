use axum::{
    extract::{Path, Query, State},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use crate::{
    api::{get_page_number_and_size, get_page_size, ServiceState},
    persistence::{
        api::{
            block::CrystalBlockAPIPostgreSQLStorage as _, event::CrystalEventAPIPostgreSQLStorage,
        },
        CrystalPostgreSQLStorage,
    },
    types::api::{
        dto::{
            pagination::{CursorPaginationData, PaginationData},
            request::{
                block::BlockReference,
                event::{
                    BlockEventQuery, EventCursorPayload, EventCursorPosition, EventQuery,
                    IncludeEventArgsParam,
                },
            },
            response::{
                error::{BadRequest, InternalServerError, NotFound, TooManyRequests},
                event::{CursorEventList, EventArgs, EventDTO, EventList, PaginatedEventList},
            },
        },
        error::APIError,
    },
};

#[utoipa::path(
    get,
    path = "/events",
    tag = "event",
    summary = "Get events",
    description = "Returns all events from the database that satisfy the query parameters. It will return a paginated response, ordered descending by block number and ascending event index.",
    params(EventQuery),
    responses(
        (
            status = 200,
            response = CursorEventList,
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
pub(crate) async fn get_events(
    State(state): State<ServiceState>,
    Query(query): Query<EventQuery>,
) -> Result<Json<CursorEventList>, APIError> {
    let (cursor_position, query) = if let Some(cursor) = query.next_cursor {
        // TODO validate that no other query params are set
        let decoded = URL_SAFE_NO_PAD.decode(cursor)?;
        let cursor_payload: EventCursorPayload = serde_json::from_slice(&decoded)?;
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

    // max block number, block hash, event index
    let rows = state
        .postgres
        .get_events(
            cursor_position,
            min_block_number,
            max_block_number,
            &query.pallet_name,
            &query.event_name,
            page_size,
            query.include_args,
        )
        .await?;
    let mut data: Vec<EventDTO> = Vec::new();
    for row in rows.iter() {
        data.push(row.into());
    }
    let next_cursor = if data.len() < page_size as usize {
        None
    } else if let Some(last_event) = data.last() {
        let cursor_payload = EventCursorPayload {
            cursor_position: EventCursorPosition {
                block_number: last_event.block_number,
                block_hash_hex: last_event.block_hash.0.clone(),
                index: last_event.index,
            },
            query,
        };
        let cursor = serde_json::to_string(&cursor_payload)?;
        let cursor_encoded = URL_SAFE_NO_PAD.encode(cursor.as_bytes());
        Some(cursor_encoded)
    } else {
        None
    };
    let response = CursorEventList {
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
    path = "/blocks/{block_ref}/events",
    tag = "event",
    summary = "Get block events",
    description = "If a hash is passed, returns the events for the matching block. If a number is passed, gives the events for the block with that number - could be multiple blocks if there's a pruned block in that slot.",
    params(
        (
            "block_ref" = String,
            Path,
            description = "Block reference for the events. Either a block number (integer ≥ 0), or a block hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
        BlockEventQuery,
    ),
    responses(
        (
            status = 200,
            response = PaginatedEventList,
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
pub(crate) async fn get_events_by_block_reference(
    State(state): State<ServiceState>,
    Path(block_reference): Path<String>,
    Query(query): Query<BlockEventQuery>,
) -> Result<Json<PaginatedEventList>, APIError> {
    let (page, page_size) =
        get_page_number_and_size(query.page, query.page_size, query.include_args)?;
    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(block_number)) => {
            if !state.postgres.block_exists_by_number(block_number).await? {
                return Err(APIError::BlockNotFoundWithNumber(block_number));
            }
            let (total_count, rows) = tokio::try_join!(
                state.postgres.get_event_count_by_block_number(
                    block_number,
                    &query.pallet_name,
                    &query.pallet_event_name,
                ),
                state.postgres.get_events_by_block_number(
                    block_number,
                    &query.pallet_name,
                    &query.pallet_event_name,
                    page,
                    page_size,
                    query.include_args,
                ),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.into());
            }
            let response = PaginatedEventList {
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
                state.postgres.get_event_count_by_block_hash(
                    &block_hash,
                    &query.pallet_name,
                    &query.pallet_event_name,
                ),
                state.postgres.get_events_by_block_hash(
                    &block_hash,
                    &query.pallet_name,
                    &query.pallet_event_name,
                    page,
                    page_size,
                    query.include_args,
                ),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.into());
            }
            let response = PaginatedEventList {
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
    path = "/blocks/{block_ref}/events/{event_index}",
    tag = "block",
    summary = "Get block events by index",
    description = "Returns the events in the given block at the given index. It can return multiple events if a number is passed and there's a pruned block in that slot.",
    params(
        (
            "block_ref" = String,
            Path,
            description = "Block reference for the event(s). Either a block number (integer ≥ 0), or a block hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
        (
            "event_index" = u32,
            Path,
            description = "0-based index for the event(s) in the block(s).",
        ),
        IncludeEventArgsParam,
    ),
    responses(
        (
            status = 200,
            response = EventList,
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
pub(crate) async fn get_events_by_block_reference_and_index(
    State(state): State<ServiceState>,
    Path((block_reference, index)): Path<(String, u32)>,
    Query(query): Query<IncludeEventArgsParam>,
) -> Result<Json<EventList>, APIError> {
    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(block_number)) => {
            if !state.postgres.block_exists_by_number(block_number).await? {
                return Err(APIError::BlockNotFoundWithNumber(block_number));
            }
            let rows = state
                .postgres
                .get_events_by_block_number_and_index(block_number, index, query.include_args)
                .await?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.into());
            }
            Ok(Json(EventList(data)))
        }
        Ok(BlockReference::Hash(block_hash)) => {
            if !state.postgres.block_exists_by_hash(&block_hash).await? {
                return Err(APIError::BlockNotFoundWithHash(block_hash));
            }
            let data = if let Some(row) = &state
                .postgres
                .get_event_by_block_hash_and_index(&block_hash, index, query.include_args)
                .await?
            {
                vec![row.into()]
            } else {
                vec![]
            };
            Ok(Json(EventList(data)))
        }
        Err(message) => Err(APIError::BadRequest(message)),
    }
}

#[utoipa::path(
    get,
    path = "/blocks/{block_ref}/extrinsics/{extrinsic_index}/events",
    tag = "event",
    summary = "Get block extrinsic events",
    description = "Returns the events for extrinsic in a block by block reference and 0-based extrinsic index.",
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
        BlockEventQuery,
    ),
    responses(
        (
            status = 200,
            response = PaginatedEventList,
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
pub(crate) async fn get_events_by_block_reference_and_extrinsic_index(
    State(state): State<ServiceState>,
    Path((block_reference, extrinsic_index)): Path<(String, u32)>,
    Query(query): Query<BlockEventQuery>,
) -> Result<Json<PaginatedEventList>, APIError> {
    let (page, page_size) =
        get_page_number_and_size(query.page, query.page_size, query.include_args)?;
    match BlockReference::try_from(block_reference.as_str()) {
        Ok(BlockReference::Number(block_number)) => {
            if !state.postgres.block_exists_by_number(block_number).await? {
                return Err(APIError::BlockNotFoundWithNumber(block_number));
            }
            let (total_count, rows) = tokio::try_join!(
                state
                    .postgres
                    .get_event_count_by_block_number_and_extrinsic_index(
                        block_number,
                        extrinsic_index,
                        &query.pallet_name,
                        &query.pallet_event_name,
                    ),
                state
                    .postgres
                    .get_events_by_block_number_and_extrinsic_index(
                        block_number,
                        extrinsic_index,
                        &query.pallet_name,
                        &query.pallet_event_name,
                        page,
                        page_size,
                        query.include_args,
                    ),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.into());
            }
            let response = PaginatedEventList {
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
                    .get_event_count_by_block_hash_and_extrinsic_index(
                        &block_hash,
                        extrinsic_index,
                        &query.pallet_name,
                        &query.pallet_event_name,
                    ),
                state.postgres.get_events_by_block_hash_and_extrinsic_index(
                    &block_hash,
                    extrinsic_index,
                    &query.pallet_name,
                    &query.pallet_event_name,
                    page,
                    page_size,
                    query.include_args,
                ),
            )?;
            let mut data = Vec::new();
            for row in rows.iter() {
                data.push(row.into());
            }
            let response = PaginatedEventList {
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
    path = "/extrinsics/{extrinsic_hash}/events",
    tag = "event",
    summary = "Get extrinsic events",
    description = "Returns the events in an extrinsic by extrinsic hash.",
    params(
        (
            "extrinsic_hash" = String,
            Path,
            description = "Extrinsic hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:0x)?[0-9a-fA-F]{64}$",
        ),
        BlockEventQuery,
    ),
    responses(
        (
            status = 200,
            response = PaginatedEventList,
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
pub(crate) async fn get_events_by_extrinsic_hash(
    State(state): State<ServiceState>,
    Path(extrinsic_hash): Path<String>,
    Query(query): Query<BlockEventQuery>,
) -> Result<Json<PaginatedEventList>, APIError> {
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
        state.postgres.get_event_count_by_extrinsic_hash(
            &extrinsic_hash,
            &query.pallet_name,
            &query.pallet_event_name,
        ),
        state.postgres.get_events_by_extrinsic_hash(
            &extrinsic_hash,
            &query.pallet_name,
            &query.pallet_event_name,
            page,
            page_size,
            query.include_args,
        ),
    )?;
    let mut data = Vec::new();
    for row in rows.iter() {
        data.push(row.into());
    }
    let response = PaginatedEventList {
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
    path = "/events/{event_hash}",
    tag = "event",
    summary = "Get event by hash",
    description = "Returns the event by its hash.",
    params(
        (
            "event_hash" = String,
            Path,
            description = "Event hash in hex (with or without `0x` prefix, case-insensitive).",
            pattern = r"^(?:\d+|(0x)?[a-f0-9A-F]{64})$",
        ),
        IncludeEventArgsParam,
    ),
    responses(
        (
            status = 200,
            headers(
                ("X-RateLimit-Limit" = u32),
                ("X-RateLimit-Remaining" = u32),
            ),
            description = "Event with the given hash.",
            body = EventDTO,
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
pub(crate) async fn get_event_by_hash(
    State(state): State<ServiceState>,
    Path(event_hash): Path<String>,
    Query(query): Query<IncludeEventArgsParam>,
) -> Result<Json<EventDTO>, APIError> {
    let event_hash = match hex::decode(event_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid event hash: {e}"))),
    };
    if let Some(row) = &state
        .postgres
        .get_event_by_hash(&event_hash, query.include_args)
        .await?
    {
        Ok(Json(row.into()))
    } else {
        Err(APIError::EventNotFoundWithHash(event_hash))
    }
}

#[utoipa::path(
    get,
    path = "/events/{event_hash}/args",
    tag = "event",
    summary = "Get event arguments",
    description = "Returns the arguments of a runtime call by its hash.",
    params(
        (
            "event_hash" = String,
            Path,
            description = "Event hash in hex (with or without `0x` prefix, case-insensitive).",
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
            description = "Arguments for the runtime event with the given hash.",
            body = EventArgs,
            example = json!({
                "hash": "0x2c923bb54d06dfb649aaaf1c198eb1af9e19ec52b8e90267984496c128ee7adc",
                "args": {
                    "to": "0x967cccc1ff3d1f37b9e6c8a39d8ba72ad85d35e19cc0717a72f1a21037606144",
                    "from": "0x96b4be4ad947987922c88449866e738b4f4d09dece5157d2c3ac9477d8c6512e",
                    "amount": "171162271"
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
pub(crate) async fn get_event_args_by_hash(
    State(state): State<ServiceState>,
    Path(event_hash): Path<String>,
) -> Result<Json<EventArgs>, APIError> {
    let event_hash = match hex::decode(event_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid event hash: {e}"))),
    };
    if let Some(args) = state.postgres.get_event_args_by_hash(&event_hash).await? {
        Ok(Json(EventArgs(args)))
    } else {
        Err(APIError::EventNotFoundWithHash(event_hash))
    }
}
