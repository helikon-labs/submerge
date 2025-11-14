use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    api::ServiceState,
    persistence::{api::genesis::CrystalMetadataAPIPostgreSQLStorage, CrystalPostgreSQLStorage},
    types::api::{
        dto::{
            genesis::GenesisRecordDTO,
            pagination::{PagedResponse, PaginationData, PaginationQuery},
            trace::TraceType,
        },
        error::APIError,
    },
};

const MAX_PAGE_SIZE: u64 = 100;
const DEFAULT_PAGE_SIZE: u64 = 50;

pub(crate) async fn get_genesis_records(
    State(state): State<ServiceState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PagedResponse<GenesisRecordDTO>>, APIError> {
    let page = query.get_page()?;
    let page_size = query.get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;
    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_genesis_record_count(),
        state.postgres.get_genesis_record_rows(page, page_size),
    )?;
    let response = PagedResponse {
        pagination: PaginationData {
            page,
            page_size,
            total: total_count,
        },
        data: rows
            .iter()
            .map(|row| GenesisRecordDTO {
                key_prefix: format!("0x{}", hex::encode(&row.key_prefix)),
                key_params: row
                    .key_params
                    .as_deref()
                    .map(|key_params| format!("0x{}", hex::encode(key_params))),
                value: format!("0x{}", hex::encode(&row.value)),
                record_type: if row.metadata_storage_item_id.is_some() {
                    Some(TraceType::StorageItem)
                } else if row.is_known_key {
                    Some(TraceType::KnownKey)
                } else {
                    None
                },
                pallet_index: row.pallet_index.map(|index| index as u32),
                pallet_name: row.pallet_name.clone(),
                pallet_storage_item_index: row.pallet_storage_item_index.map(|index| index as u32),
                pallet_storage_item_name: row.pallet_storage_item_name.clone(),
            })
            .collect(),
    };
    Ok(Json(response))
}
