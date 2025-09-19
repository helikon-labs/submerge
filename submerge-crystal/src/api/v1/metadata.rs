use axum::{
    extract::{Path, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    api::ServiceState,
    persistence::api::CrystalAPIPostgreSQLStorage as _,
    types::api::{
        dto::metadata::{
            Metadata, MetadataPallet, MetadataPalletCall, MetadataPalletConstant,
            MetadataPalletError, MetadataPalletEvent, MetadataPalletStorageItem,
        },
        error::APIError,
    },
};
use serde_json::Value as JSONValue;

fn parse_spec_version(spec_version: &str) -> Result<u32, APIError> {
    spec_version.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid spec_version: must be a positive integer.".to_string())
    })
}

fn parse_pallet_index(pallet_index: &str) -> Result<u32, APIError> {
    pallet_index.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid pallet index: must be a positive integer.".to_string())
    })
}

pub(crate) async fn get_metadata_list(
    State(state): State<ServiceState>,
) -> Result<Json<Vec<Metadata>>, APIError> {
    let rows = state.postgres.get_metadata_list().await?;
    Ok(Json(rows))
}

pub(crate) async fn get_metadata_json(
    State(state): State<ServiceState>,
    Path(spec_version): Path<String>,
) -> Result<Json<JSONValue>, APIError> {
    let spec_version = parse_spec_version(&spec_version)?;
    let maybe_metadata_json = state.postgres.get_metadata_json(spec_version).await?;
    match maybe_metadata_json {
        Some(data) => Ok(Json(data)),
        None => Err(APIError::MetadataNotFound(spec_version)),
    }
}

pub(crate) async fn get_metadata_hex(
    State(state): State<ServiceState>,
    Path(spec_version): Path<String>,
) -> Result<Response, APIError> {
    let spec_version = parse_spec_version(&spec_version)?;
    let maybe_metadata_bytes = state.postgres.get_metadata_bytes(spec_version).await?;
    if let Some(metadata_bytes) = maybe_metadata_bytes {
        let hex_string = format!("0x{}", hex::encode(metadata_bytes));
        Ok(([(header::CONTENT_TYPE, "text/plain")], hex_string).into_response())
    } else {
        Err(APIError::MetadataNotFound(spec_version))
    }
}

pub(crate) async fn get_metadata_pallets(
    State(state): State<ServiceState>,
    Path(spec_version): Path<String>,
) -> Result<Json<Vec<MetadataPallet>>, APIError> {
    let spec_version = parse_spec_version(&spec_version)?;
    if state.postgres.metadata_exists(spec_version).await? {
        let metadata_pallets = state.postgres.get_metadata_pallets(spec_version).await?;
        Ok(Json(metadata_pallets))
    } else {
        Err(APIError::MetadataNotFound(spec_version))
    }
}

pub(crate) async fn get_metadata_pallet_calls(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(String, String)>,
) -> Result<Json<Vec<MetadataPalletCall>>, APIError> {
    let spec_version = parse_spec_version(&spec_version)?;
    let pallet_index = parse_pallet_index(&pallet_index)?;
    if !state.postgres.metadata_exists(spec_version).await? {
        Err(APIError::MetadataNotFound(spec_version))
    } else if !state
        .postgres
        .metadata_pallet_exists(spec_version, pallet_index)
        .await?
    {
        Err(APIError::MetadataPalletNotFound(spec_version, pallet_index))
    } else {
        let metadata_pallet_calls = state
            .postgres
            .get_metadata_pallet_calls(spec_version, pallet_index)
            .await?;
        Ok(Json(metadata_pallet_calls))
    }
}

pub(crate) async fn get_metadata_pallet_constants(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(String, String)>,
) -> Result<Json<Vec<MetadataPalletConstant>>, APIError> {
    let spec_version = parse_spec_version(&spec_version)?;
    let pallet_index = parse_pallet_index(&pallet_index)?;
    if !state.postgres.metadata_exists(spec_version).await? {
        Err(APIError::MetadataNotFound(spec_version))
    } else if !state
        .postgres
        .metadata_pallet_exists(spec_version, pallet_index)
        .await?
    {
        Err(APIError::MetadataPalletNotFound(spec_version, pallet_index))
    } else {
        let metadata_pallet_constants = state
            .postgres
            .get_metadata_pallet_constants(spec_version, pallet_index)
            .await?;
        Ok(Json(metadata_pallet_constants))
    }
}

pub(crate) async fn get_metadata_pallet_errors(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(String, String)>,
) -> Result<Json<Vec<MetadataPalletError>>, APIError> {
    let spec_version = parse_spec_version(&spec_version)?;
    let pallet_index = parse_pallet_index(&pallet_index)?;
    if !state.postgres.metadata_exists(spec_version).await? {
        Err(APIError::MetadataNotFound(spec_version))
    } else if !state
        .postgres
        .metadata_pallet_exists(spec_version, pallet_index)
        .await?
    {
        Err(APIError::MetadataPalletNotFound(spec_version, pallet_index))
    } else {
        let metadata_pallet_errors = state
            .postgres
            .get_metadata_pallet_errors(spec_version, pallet_index)
            .await?;
        Ok(Json(metadata_pallet_errors))
    }
}

pub(crate) async fn get_metadata_pallet_events(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(String, String)>,
) -> Result<Json<Vec<MetadataPalletEvent>>, APIError> {
    let spec_version = parse_spec_version(&spec_version)?;
    let pallet_index = parse_pallet_index(&pallet_index)?;
    if !state.postgres.metadata_exists(spec_version).await? {
        Err(APIError::MetadataNotFound(spec_version))
    } else if !state
        .postgres
        .metadata_pallet_exists(spec_version, pallet_index)
        .await?
    {
        Err(APIError::MetadataPalletNotFound(spec_version, pallet_index))
    } else {
        let metadata_pallet_events = state
            .postgres
            .get_metadata_pallet_events(spec_version, pallet_index)
            .await?;
        Ok(Json(metadata_pallet_events))
    }
}

pub(crate) async fn get_metadata_pallet_storage_items(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(String, String)>,
) -> Result<Json<Vec<MetadataPalletStorageItem>>, APIError> {
    let spec_version = parse_spec_version(&spec_version)?;
    let pallet_index = parse_pallet_index(&pallet_index)?;
    if !state.postgres.metadata_exists(spec_version).await? {
        Err(APIError::MetadataNotFound(spec_version))
    } else if !state
        .postgres
        .metadata_pallet_exists(spec_version, pallet_index)
        .await?
    {
        Err(APIError::MetadataPalletNotFound(spec_version, pallet_index))
    } else {
        let metadata_pallet_storage_items = state
            .postgres
            .get_metadata_pallet_storage_items(spec_version, pallet_index)
            .await?;
        Ok(Json(metadata_pallet_storage_items))
    }
}
