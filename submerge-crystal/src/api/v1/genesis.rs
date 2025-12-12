use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    api::{get_page_number_and_size, ServiceState},
    persistence::{api::genesis::CrystalMetadataAPIPostgreSQLStorage, CrystalPostgreSQLStorage},
    types::api::{
        dto::{
            pagination::PaginationData,
            request::genesis::GenesisRecordQuery,
            response::{
                error::{BadRequest, InternalServerError, TooManyRequests},
                genesis::{GenesisRecordDTO, PaginatedGenesisRecordList},
                hex::HexString,
            },
        },
        error::APIError,
    },
};

#[utoipa::path(
    get,
    path = "/genesis",
    tag = "genesis",
    summary = "Get genesis records",
    description = "Returns a paginated list of all genesis storage records.",
    params(GenesisRecordQuery),
    responses(
        (
            status = 200,
            response = PaginatedGenesisRecordList,
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
pub(crate) async fn get_genesis_records(
    State(state): State<ServiceState>,
    Query(query): Query<GenesisRecordQuery>,
) -> Result<Json<PaginatedGenesisRecordList>, APIError> {
    let (page, page_size) = get_page_number_and_size(query.page, query.page_size)?;
    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_genesis_record_count(),
        state.postgres.get_genesis_record_rows(page, page_size),
    )?;
    let mut data = Vec::new();
    for row in rows.iter() {
        data.push(GenesisRecordDTO {
            key_prefix: HexString(format!("0x{}", hex::encode(&row.key_prefix))),
            key_params: row
                .key_params
                .as_deref()
                .map(|key_params| HexString(format!("0x{}", hex::encode(key_params)))),
            value: HexString(format!("0x{}", hex::encode(&row.value))),
            known_key: if row.is_known_key {
                Some(String::from_utf8(row.key_prefix.clone())?)
            } else {
                None
            },
            pallet_index: row.pallet_index.map(|index| index as u32),
            pallet_name: row.pallet_name.clone(),
            pallet_storage_item_index: row.pallet_storage_item_index.map(|index| index as u32),
            pallet_storage_item_name: row.pallet_storage_item_name.clone(),
        });
    }

    let response = PaginatedGenesisRecordList {
        pagination: PaginationData {
            page,
            page_size,
            total: total_count,
        },
        data,
    };
    Ok(Json(response))
}
