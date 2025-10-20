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
                id: row.id as u64,
                key: format!("0x{}", hex::encode(&row.key)),
                key_prefix: format!("0x{}", hex::encode(&row.key_prefix)),
                value: format!("0x{}", hex::encode(&row.value)),
            })
            .collect(),
    };
    Ok(Json(response))
}
