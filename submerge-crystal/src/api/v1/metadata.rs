use crate::{
    api::{APIResult, ServiceState},
    persistence::api::CrystalAPIPostgreSQLStorage,
    types::api::error::APIError,
};
use actix_web::{get, http::header, web, HttpResponse};
use serde::Deserialize;

#[get("/metadata")]
pub(crate) async fn get_metadata_list(state: web::Data<ServiceState>) -> APIResult {
    let rows = state.postgres.get_metadata_list().await?;
    Ok(HttpResponse::Ok().json(rows))
}

#[derive(Deserialize)]
pub(crate) struct MetadataSpecVersionPathParameter {
    spec_version: String,
}

#[get("/metadata/{spec_version}/json")]
pub(crate) async fn get_metadata_json(
    path: web::Path<MetadataSpecVersionPathParameter>,
    state: web::Data<ServiceState>,
) -> APIResult {
    let spec_version = path.spec_version.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid spec_version: must be a positive integer.".to_string())
    })?;
    let maybe_metadata_json = state.postgres.get_metadata_json(spec_version).await?;
    if let Some(metadata_json) = maybe_metadata_json {
        Ok(HttpResponse::Ok().json(metadata_json))
    } else {
        Err(APIError::MetadataNotFound(spec_version))
    }
}

#[get("/metadata/{spec_version}/hex")]
pub(crate) async fn get_metadata_hex(
    path: web::Path<MetadataSpecVersionPathParameter>,
    state: web::Data<ServiceState>,
) -> APIResult {
    let spec_version = path.spec_version.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid spec_version: must be a positive integer.".to_string())
    })?;
    let maybe_metadata_bytes = state.postgres.get_metadata_bytes(spec_version).await?;
    if let Some(metadata_bytes) = maybe_metadata_bytes {
        Ok(HttpResponse::Ok()
            .content_type(header::ContentType::plaintext())
            .body(format!("0x{}", hex::encode(metadata_bytes))))
    } else {
        Err(APIError::MetadataNotFound(spec_version))
    }
}

#[get("/metadata/{spec_version}/pallets")]
pub(crate) async fn get_metadata_pallets(
    path: web::Path<MetadataSpecVersionPathParameter>,
    state: web::Data<ServiceState>,
) -> APIResult {
    let spec_version = path.spec_version.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid spec_version: must be a positive integer.".to_string())
    })?;
    if state.postgres.metadata_exists(spec_version).await? {
        let metadata_pallets = state.postgres.get_metadata_pallets(spec_version).await?;
        Ok(HttpResponse::Ok().json(metadata_pallets))
    } else {
        Err(APIError::MetadataNotFound(spec_version))
    }
}

#[derive(Deserialize)]
pub(crate) struct MetadataSpecVersionItemIndexPathParameter {
    spec_version: String,
    index: String,
}

#[get("/metadata/{spec_version}/pallets/{index}/calls")]
pub(crate) async fn get_metadata_pallet_calls(
    path: web::Path<MetadataSpecVersionItemIndexPathParameter>,
    state: web::Data<ServiceState>,
) -> APIResult {
    let spec_version = path.spec_version.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid spec_version: must be a positive integer.".to_string())
    })?;
    let index = path.index.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid index: must be a positive integer.".to_string())
    })?;
    if !state.postgres.metadata_exists(spec_version).await? {
        Err(APIError::MetadataNotFound(spec_version))
    } else if !state
        .postgres
        .metadata_pallet_exists(spec_version, index)
        .await?
    {
        Err(APIError::MetadataPalletNotFound(spec_version, index))
    } else {
        let metadata_pallet_calls = state
            .postgres
            .get_metadata_pallet_calls(spec_version, index)
            .await?;
        Ok(HttpResponse::Ok().json(metadata_pallet_calls))
    }
}

#[get("/metadata/{spec_version}/pallets/{index}/constants")]
pub(crate) async fn get_metadata_pallet_constants(
    path: web::Path<MetadataSpecVersionItemIndexPathParameter>,
    state: web::Data<ServiceState>,
) -> APIResult {
    let spec_version = path.spec_version.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid spec_version: must be a positive integer.".to_string())
    })?;
    let index = path.index.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid index: must be a positive integer.".to_string())
    })?;
    if !state.postgres.metadata_exists(spec_version).await? {
        Err(APIError::MetadataNotFound(spec_version))
    } else if !state
        .postgres
        .metadata_pallet_exists(spec_version, index)
        .await?
    {
        Err(APIError::MetadataPalletNotFound(spec_version, index))
    } else {
        let metadata_pallet_calls = state
            .postgres
            .get_metadata_pallet_constants(spec_version, index)
            .await?;
        Ok(HttpResponse::Ok().json(metadata_pallet_calls))
    }
}

#[get("/metadata/{spec_version}/pallets/{index}/errors")]
pub(crate) async fn get_metadata_pallet_errors(
    path: web::Path<MetadataSpecVersionItemIndexPathParameter>,
    state: web::Data<ServiceState>,
) -> APIResult {
    let spec_version = path.spec_version.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid spec_version: must be a positive integer.".to_string())
    })?;
    let index = path.index.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid index: must be a positive integer.".to_string())
    })?;
    if !state.postgres.metadata_exists(spec_version).await? {
        Err(APIError::MetadataNotFound(spec_version))
    } else if !state
        .postgres
        .metadata_pallet_exists(spec_version, index)
        .await?
    {
        Err(APIError::MetadataPalletNotFound(spec_version, index))
    } else {
        let metadata_pallet_errors = state
            .postgres
            .get_metadata_pallet_errors(spec_version, index)
            .await?;
        Ok(HttpResponse::Ok().json(metadata_pallet_errors))
    }
}

#[get("/metadata/{spec_version}/pallets/{index}/events")]
pub(crate) async fn get_metadata_pallet_events(
    path: web::Path<MetadataSpecVersionItemIndexPathParameter>,
    state: web::Data<ServiceState>,
) -> APIResult {
    let spec_version = path.spec_version.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid spec_version: must be a positive integer.".to_string())
    })?;
    let index = path.index.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid index: must be a positive integer.".to_string())
    })?;
    if !state.postgres.metadata_exists(spec_version).await? {
        Err(APIError::MetadataNotFound(spec_version))
    } else if !state
        .postgres
        .metadata_pallet_exists(spec_version, index)
        .await?
    {
        Err(APIError::MetadataPalletNotFound(spec_version, index))
    } else {
        let metadata_pallet_events = state
            .postgres
            .get_metadata_pallet_events(spec_version, index)
            .await?;
        Ok(HttpResponse::Ok().json(metadata_pallet_events))
    }
}

#[get("/metadata/{spec_version}/pallets/{index}/storage")]
pub(crate) async fn get_metadata_pallet_storage_items(
    path: web::Path<MetadataSpecVersionItemIndexPathParameter>,
    state: web::Data<ServiceState>,
) -> APIResult {
    let spec_version = path.spec_version.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid spec_version: must be a positive integer.".to_string())
    })?;
    let index = path.index.parse::<u32>().map_err(|_| {
        APIError::BadRequest("Invalid index: must be a positive integer.".to_string())
    })?;
    if !state.postgres.metadata_exists(spec_version).await? {
        Err(APIError::MetadataNotFound(spec_version))
    } else if !state
        .postgres
        .metadata_pallet_exists(spec_version, index)
        .await?
    {
        Err(APIError::MetadataPalletNotFound(spec_version, index))
    } else {
        let metadata_pallet_storage_items = state
            .postgres
            .get_metadata_pallet_storage_items(spec_version, index)
            .await?;
        Ok(HttpResponse::Ok().json(metadata_pallet_storage_items))
    }
}
