use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    api::ServiceState,
    persistence::{api::genesis::CrystalMetadataAPIPostgreSQLStorage, CrystalPostgreSQLStorage},
    types::api::{
        dto::{
            genesis::GenesisRecord,
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
) -> Result<Json<PagedResponse<GenesisRecord>>, APIError> {
    let page_number = query.get_page_number()?;
    let page_size = query.get_page_size(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE)?;
    let total_count = state.postgres.get_genesis_record_count().await?;
    let rows = state
        .postgres
        .get_genesis_record_rows(page_number, page_size)
        .await?;
    let response = PagedResponse {
        pagination: PaginationData {
            page_number,
            page_size,
            total_count,
        },
        data: rows
            .iter()
            .map(|row| GenesisRecord {
                id: row.id as u64,
                key: format!("0x{}", row.key),
                value: format!("0x{}", row.value),
            })
            .collect(),
    };
    Ok(Json(response))
}
