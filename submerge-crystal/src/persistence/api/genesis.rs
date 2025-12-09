use sqlx::FromRow;
use submerge_persistence::postgres::PostgreSQLStorage;

#[derive(Clone, Debug, FromRow)]
pub struct GenesisRecordRow {
    #[allow(dead_code)]
    pub id: i32,
    pub key_prefix: Vec<u8>,
    pub key_params: Option<Vec<u8>>,
    pub value: Vec<u8>,
    pub is_known_key: bool,
    pub pallet_index: Option<i32>,
    pub pallet_name: Option<String>,
    pub pallet_storage_item_index: Option<i32>,
    pub pallet_storage_item_name: Option<String>,
}

pub(crate) trait CrystalMetadataAPIPostgreSQLStorage {
    async fn get_genesis_record_rows(
        &self,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<GenesisRecordRow>>;
}

impl CrystalMetadataAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_genesis_record_rows(
        &self,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<Vec<GenesisRecordRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<GenesisRecordRow> = sqlx::query_as(
            r#"
            SELECT
                G.id, G.key_prefix, G.key_params, G.value, G.is_known_key,
	            MP.index AS pallet_index, MP.name AS pallet_name,
                MPSI.index AS pallet_storage_item_index, MPSI.name AS pallet_storage_item_name
            FROM genesis G
            LEFT JOIN metadata_storage_item MPSI ON G.metadata_storage_item_id = MPSI.id
            LEFT JOIN metadata_pallet MP ON MP.id = MPSI.pallet_id
            ORDER BY G.id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }
}
