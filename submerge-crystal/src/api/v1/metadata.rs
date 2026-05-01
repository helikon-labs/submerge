use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    api::{get_page_number_and_size, ServiceState},
    persistence::{
        api::metadata::CrystalMetadataAPIPostgreSQLStorage as _, CrystalPostgreSQLStorage as _,
    },
    types::api::{
        dto::{
            pagination::{PaginationData, PaginationQuery},
            response::{
                error::{BadRequest, InternalServerError, NotFound, TooManyRequests},
                hex::HexString,
                metadata::{
                    MetadataDTO, MetadataJSON, MetadataPalletCallList, MetadataPalletConstantList,
                    MetadataPalletDTO, MetadataPalletErrorList, MetadataPalletEventList,
                    MetadataPalletStorageItemList, MetadataPalletSummaryList,
                    PaginatedMetadataList,
                },
            },
        },
        error::APIError,
    },
};

#[utoipa::path(
    get,
    path = "/metadata",
    tag = "metadata",
    summary = "Get metadata list",
    description = "Returns a list of metadata summaries for all runtime versions.",
    params(PaginationQuery),
    responses(
        (
            status = 200,
            response = PaginatedMetadataList,
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
pub(crate) async fn get_metadata_list(
    State(state): State<ServiceState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedMetadataList>, APIError> {
    let (page, page_size) = get_page_number_and_size(query.page, query.page_size, false)?;
    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_metadata_count(),
        state.postgres.get_metadata_list(page, page_size),
    )?;
    let count = rows.len() as u32;
    Ok(Json(PaginatedMetadataList {
        pagination: PaginationData {
            count,
            page,
            page_size,
            total: total_count,
        },
        data: rows,
    }))
}

#[utoipa::path(
    get,
    path = "/metadata/{spec_version}",
    tag = "metadata",
    summary = "Get metadata by spec version",
    description = "Returns the metadata by its spec version.",
    params(
        (
            "spec_version" = u32,
            Path,
            description = "Metadata spec version.",
            example = 1003,
        ),
    ),
    responses(
        (
            status = 200,
            headers(
                ("X-RateLimit-Limit" = u32),
                ("X-RateLimit-Remaining" = u32),
            ),
            description = "Full metadata with the given spec version.",
            body = MetadataDTO,
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
pub(crate) async fn get_metadata(
    State(state): State<ServiceState>,
    Path(spec_version): Path<u32>,
) -> Result<Json<MetadataDTO>, APIError> {
    let Some(metadata) = state.postgres.get_metadata_dto(spec_version).await? else {
        return Err(APIError::MetadataNotFound(spec_version));
    };
    let mut metadata = MetadataDTO {
        spec_version,
        metadata_version: metadata.metadata_version,
        pallets: Vec::new(),
    };
    for pallet in state
        .postgres
        .get_metadata_pallet_summaries(spec_version)
        .await?
        .iter()
    {
        let pallet_full = MetadataPalletDTO {
            index: pallet.index,
            name: pallet.name.clone(),
            calls: state
                .postgres
                .get_metadata_calls(spec_version, pallet.index)
                .await?,
            constants: state
                .postgres
                .get_metadata_constants(spec_version, pallet.index)
                .await?,
            errors: state
                .postgres
                .get_metadata_errors(spec_version, pallet.index)
                .await?,
            events: state
                .postgres
                .get_metadata_events(spec_version, pallet.index)
                .await?,
            storage_items: state
                .postgres
                .get_metadata_storage_items(spec_version, pallet.index)
                .await?,
        };
        metadata.pallets.push(pallet_full);
    }
    Ok(Json(metadata))
}

#[utoipa::path(
    get,
    path = "/metadata/{spec_version}/json",
    tag = "metadata",
    summary = "Get metadata JSON",
    description = "Returns the original Substrate runtime metadata by its spec version in JSON representation.",
    params(
        (
            "spec_version" = u32,
            Path,
            description = "Metadata spec version.",
            example = 1003,
        ),
    ),
    responses(
        (
            status = 200,
            headers(
                ("X-RateLimit-Limit" = u32),
                ("X-RateLimit-Remaining" = u32),
            ),
            description = "Full metadata with the given spec version.",
            body = MetadataJSON,
            example = json!([
                1635018093,
                {
                    "V14": {
                        "ty": 396,
                        "extrinsic": {
                            "ty": 380,
                            "version": 4,
                            "signed_extensions": [
                                {
                                    "additional_signed": 122,
                                    "identifier": "CheckNonZeroSender",
                                    "ty": 385
                                },
                                {
                                    "additional_signed": 4,
                                    "identifier": "CheckSpecVersion",
                                    "ty": 386
                                },
                                {
                                    "additional_signed": 4,
                                    "identifier": "CheckTxVersion",
                                    "ty": 387
                                },
                                {
                                    "additional_signed": 13,
                                    "identifier": "CheckGenesis",
                                    "ty": 388
                                },
                                {
                                    "additional_signed": 13,
                                    "identifier": "CheckMortality",
                                    "ty": 389
                                },
                                {
                                    "additional_signed": 122,
                                    "identifier": "CheckNonce",
                                    "ty": 391
                                },
                                {
                                    "additional_signed": 122,
                                    "identifier": "CheckWeight",
                                    "ty": 392
                                },
                                {
                                    "additional_signed": 122,
                                    "identifier": "ChargeTransactionPayment",
                                    "ty": 393
                                },
                                {
                                    "additional_signed": 32,
                                    "identifier": "CheckMetadataHash",
                                    "ty": 394
                                }
                            ]
                        },
                        "pallets": [],
                        "types": [],
                    }
                }
            ]),
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
pub(crate) async fn get_metadata_json(
    State(state): State<ServiceState>,
    Path(spec_version): Path<u32>,
) -> Result<Json<MetadataJSON>, APIError> {
    let maybe_metadata_json = state.postgres.get_metadata_json(spec_version).await?;
    match maybe_metadata_json {
        Some(data) => Ok(Json(MetadataJSON(data))),
        None => Err(APIError::MetadataNotFound(spec_version)),
    }
}

#[utoipa::path(
    get,
    path = "/metadata/{spec_version}/hex",
    tag = "metadata",
    summary = "Get metadata hex",
    description = "Returns the SCALE-encoded hexadecimal string for the prefixed metadata.",
    params(
        (
            "spec_version" = u32,
            Path,
            description = "Metadata spec version.",
            example = 1003,
        ),
    ),
    responses(
        (
            status = 200,
            headers(
                ("X-RateLimit-Limit" = u32),
                ("X-RateLimit-Remaining" = u32),
            ),
            description = "SCALE-encoded prefixed metadata hexadecimal string.",
            body = HexString,
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
pub(crate) async fn get_metadata_hex(
    State(state): State<ServiceState>,
    Path(spec_version): Path<u32>,
) -> Result<Response, APIError> {
    let maybe_metadata_bytes = state.postgres.get_metadata_bytes(spec_version).await?;
    if let Some(metadata_bytes) = maybe_metadata_bytes {
        let hex_string = format!("0x{}", hex::encode(metadata_bytes));
        Ok(([(header::CONTENT_TYPE, "text/plain")], hex_string).into_response())
    } else {
        Err(APIError::MetadataNotFound(spec_version))
    }
}

#[utoipa::path(
    get,
    path = "/metadata/{spec_version}/pallets",
    tag = "metadata",
    summary = "Get metadata pallets",
    description = "Returns pallets in a metadata version.",
    params(
        (
            "spec_version" = u32,
            Path,
            description = "Metadata spec version.",
            example = 1003,
        ),
    ),
    responses(
        (
            status = 200,
            response = MetadataPalletSummaryList,
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
pub(crate) async fn get_metadata_pallets(
    State(state): State<ServiceState>,
    Path(spec_version): Path<u32>,
) -> Result<Json<MetadataPalletSummaryList>, APIError> {
    if state.postgres.metadata_exists(spec_version).await? {
        let metadata_pallets = state
            .postgres
            .get_metadata_pallet_summaries(spec_version)
            .await?;
        Ok(Json(MetadataPalletSummaryList(metadata_pallets)))
    } else {
        Err(APIError::MetadataNotFound(spec_version))
    }
}

#[utoipa::path(
    get,
    path = "/metadata/{spec_version}/pallets/{pallet_index}",
    tag = "metadata",
    summary = "Get metadata pallet",
    description = "Returns the full metadata pallet by metadata spec version and pallet index.",
    params(
        (
            "spec_version" = u32,
            Path,
            description = "Metadata spec version.",
            example = 2000000,
        ),
        (
            "pallet_index" = u32,
            Path,
            description = "Metadata pallet index.",
            example = 50,
        ),
    ),
    responses(
        (
            status = 200,
            headers(
                ("X-RateLimit-Limit" = u32),
                ("X-RateLimit-Remaining" = u32),
            ),
            description = "Full metadata pallet with the given spec version and pallet index.",
            body = MetadataPalletDTO,
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
pub(crate) async fn get_metadata_pallet(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(u32, u32)>,
) -> Result<Json<MetadataPalletDTO>, APIError> {
    if !state.postgres.metadata_exists(spec_version).await? {
        return Err(APIError::MetadataNotFound(spec_version));
    }
    let Some(pallet) = state
        .postgres
        .get_metadata_pallet_dto(spec_version, pallet_index)
        .await?
    else {
        return Err(APIError::MetadataPalletNotFound(spec_version, pallet_index));
    };
    Ok(Json(MetadataPalletDTO {
        index: pallet.index,
        name: pallet.name,
        calls: state
            .postgres
            .get_metadata_calls(spec_version, pallet_index)
            .await?,
        constants: state
            .postgres
            .get_metadata_constants(spec_version, pallet_index)
            .await?,
        errors: state
            .postgres
            .get_metadata_errors(spec_version, pallet_index)
            .await?,
        events: state
            .postgres
            .get_metadata_events(spec_version, pallet_index)
            .await?,
        storage_items: state
            .postgres
            .get_metadata_storage_items(spec_version, pallet_index)
            .await?,
    }))
}

#[utoipa::path(
    get,
    path = "/metadata/{spec_version}/pallets/{pallet_index}/calls",
    tag = "metadata",
    summary = "Get metadata pallet calls",
    description = "Returns the calls in a pallet in a metadata version.",
    params(
        (
            "spec_version" = u32,
            Path,
            description = "Metadata spec version.",
            example = 1003,
        ),
        (
            "pallet_index" = u32,
            Path,
            description = "Metadata pallet index.",
            example = 50,
        ),
    ),
    responses(
        (
            status = 200,
            response = MetadataPalletCallList,
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
pub(crate) async fn get_metadata_pallet_calls(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(u32, u32)>,
) -> Result<Json<MetadataPalletCallList>, APIError> {
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
            .get_metadata_calls(spec_version, pallet_index)
            .await?;
        Ok(Json(MetadataPalletCallList(metadata_pallet_calls)))
    }
}

#[utoipa::path(
    get,
    path = "/metadata/{spec_version}/pallets/{pallet_index}/constants",
    tag = "metadata",
    summary = "Get metadata pallet constants",
    description = "Returns the constants in a pallet in a metadata version.",
    params(
        (
            "spec_version" = u32,
            Path,
            description = "Metadata spec version.",
            example = 1003,
        ),
        (
            "pallet_index" = u32,
            Path,
            description = "Metadata pallet index.",
            example = 50,
        ),
    ),
    responses(
        (
            status = 200,
            response = MetadataPalletConstantList,
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
pub(crate) async fn get_metadata_pallet_constants(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(u32, u32)>,
) -> Result<Json<MetadataPalletConstantList>, APIError> {
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
            .get_metadata_constants(spec_version, pallet_index)
            .await?;
        Ok(Json(MetadataPalletConstantList(metadata_pallet_constants)))
    }
}

#[utoipa::path(
    get,
    path = "/metadata/{spec_version}/pallets/{pallet_index}/errors",
    tag = "metadata",
    summary = "Get metadata pallet errors",
    description = "Returns the errors in a pallet in a metadata version.",
    params(
        (
            "spec_version" = u32,
            Path,
            description = "Metadata spec version.",
            example = 1003,
        ),
        (
            "pallet_index" = u32,
            Path,
            description = "Metadata pallet index.",
            example = 50,
        ),
    ),
    responses(
        (
            status = 200,
            response = MetadataPalletErrorList,
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
pub(crate) async fn get_metadata_pallet_errors(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(u32, u32)>,
) -> Result<Json<MetadataPalletErrorList>, APIError> {
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
            .get_metadata_errors(spec_version, pallet_index)
            .await?;
        Ok(Json(MetadataPalletErrorList(metadata_pallet_errors)))
    }
}

#[utoipa::path(
    get,
    path = "/metadata/{spec_version}/pallets/{pallet_index}/events",
    tag = "metadata",
    summary = "Get metadata pallet events",
    description = "Returns the events in a pallet in a metadata version.",
    params(
        (
            "spec_version" = u32,
            Path,
            description = "Metadata spec version.",
            example = 1003,
        ),
        (
            "pallet_index" = u32,
            Path,
            description = "Metadata pallet index.",
            example = 50,
        ),
    ),
    responses(
        (
            status = 200,
            response = MetadataPalletEventList,
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
pub(crate) async fn get_metadata_pallet_events(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(u32, u32)>,
) -> Result<Json<MetadataPalletEventList>, APIError> {
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
            .get_metadata_events(spec_version, pallet_index)
            .await?;
        Ok(Json(MetadataPalletEventList(metadata_pallet_events)))
    }
}

#[utoipa::path(
    get,
    path = "/metadata/{spec_version}/pallets/{pallet_index}/storage",
    tag = "metadata",
    summary = "Get metadata pallet storage items",
    description = "Returns the storage items in a pallet in a metadata version.",
    params(
        (
            "spec_version" = u32,
            Path,
            description = "Metadata spec version.",
            example = 1003,
        ),
        (
            "pallet_index" = u32,
            Path,
            description = "Metadata pallet index.",
            example = 50,
        ),
    ),
    responses(
        (
            status = 200,
            response = MetadataPalletStorageItemList,
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
pub(crate) async fn get_metadata_pallet_storage_items(
    State(state): State<ServiceState>,
    Path((spec_version, pallet_index)): Path<(u32, u32)>,
) -> Result<Json<MetadataPalletStorageItemList>, APIError> {
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
            .get_metadata_storage_items(spec_version, pallet_index)
            .await?;
        Ok(Json(MetadataPalletStorageItemList(
            metadata_pallet_storage_items,
        )))
    }
}
