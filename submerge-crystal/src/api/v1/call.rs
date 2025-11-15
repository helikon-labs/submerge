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

const DEFAULT_PAGE_SIZE: u64 = 10;
const MAX_PAGE_SIZE: u64 = 25;

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

pub(crate) async fn get_calls_by_block_reference_and_extrinsic_index(
    State(state): State<ServiceState>,
    Path((block_reference, extrinsic_index)): Path<(String, u32)>,
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

pub(crate) async fn get_calls_by_extrinsic_hash(
    State(state): State<ServiceState>,
    Path(extrinsic_hash): Path<String>,
    Query(query): Query<BlockCallQuery>,
) -> Result<Json<PagedResponse<CallDTO>>, APIError> {
    let extrinsic_hash = if let Ok(extrinsic_hash) =
        hex::decode(extrinsic_hash.trim_start_matches("0x"))
    {
        extrinsic_hash
    } else {
        return Err(APIError::BadRequest("Invalid extrinsic hash. It should be a hex string (with or without 0x prefix, case-insensitive).".to_string()));
    };
    if !state
        .postgres
        .extrinsic_exists_by_hash(&extrinsic_hash)
        .await?
    {
        return Err(APIError::ExtrinsicNotFoundWithHash(extrinsic_hash));
    }
    let page = query.pagination.get_page()?;
    let page_size = query
        .pagination
        .get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;
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

pub(crate) async fn get_call_by_hash(
    State(state): State<ServiceState>,
    Path(call_hash): Path<String>,
) -> Result<Json<CallDTO>, APIError> {
    let call_hash = match hex::decode(call_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid call hash: {e}"))),
    };
    if let Some(row) = &state.postgres.get_call_by_hash(&call_hash).await? {
        Ok(Json(row.into()))
    } else {
        Err(APIError::CallNotFoundWithHash(call_hash))
    }
}

pub(crate) async fn get_parent_call_by_hash(
    State(state): State<ServiceState>,
    Path(call_hash): Path<String>,
) -> Result<Json<CallDTO>, APIError> {
    let call_hash = match hex::decode(call_hash.trim_start_matches("0x")) {
        Ok(hash) => hash,
        Err(e) => return Err(APIError::BadRequest(format!("Invalid call hash: {e}"))),
    };
    if !state.postgres.call_exists_by_hash(&call_hash).await? {
        return Err(APIError::CallNotFoundWithHash(call_hash));
    }
    if let Some(row) = &state.postgres.get_parent_call_by_hash(&call_hash).await? {
        Ok(Json(row.into()))
    } else {
        Err(APIError::ParentCallNotFoundForCallWithHash(call_hash))
    }
}

pub(crate) async fn get_sub_calls_by_hash(
    State(state): State<ServiceState>,
    Path(call_hash): Path<String>,
    Query(query): Query<BlockCallQuery>,
) -> Result<Json<PagedResponse<CallDTO>>, APIError> {
    let call_hash = if let Ok(call_hash) = hex::decode(call_hash.trim_start_matches("0x")) {
        call_hash
    } else {
        return Err(APIError::BadRequest("Invalid call hash. It should be a hex string (with or without 0x prefix, case-insensitive).".to_string()));
    };
    if !state.postgres.call_exists_by_hash(&call_hash).await? {
        return Err(APIError::CallNotFoundWithHash(call_hash));
    }
    let page = query.pagination.get_page()?;
    let page_size = query
        .pagination
        .get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;
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
