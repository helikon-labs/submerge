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
        },
        error::APIError,
    },
};

pub(crate) async fn get_genesis_records(
    State(state): State<ServiceState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PagedResponse<GenesisRecordDTO>>, APIError> {
    let page = query.get_page()?;
    let page_size = query.get_page_size(super::DEFAULT_PAGE_SIZE, super::MAX_PAGE_SIZE)?;
    let (total_count, rows) = tokio::try_join!(
        state.postgres.get_genesis_record_count(),
        state.postgres.get_genesis_record_rows(page, page_size),
    )?;
    let mut data = Vec::new();
    for row in rows.iter() {
        data.push(GenesisRecordDTO {
            key_prefix: format!("0x{}", hex::encode(&row.key_prefix)),
            key_params: row
                .key_params
                .as_deref()
                .map(|key_params| format!("0x{}", hex::encode(key_params))),
            value: format!("0x{}", hex::encode(&row.value)),
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
