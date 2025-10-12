use sqlx::FromRow;
use submerge_persistence::postgres::PostgreSQLStorage;

#[derive(Clone, Debug, FromRow)]
pub struct GenesisRecordRow {
    pub id: i32,
    pub key: String,
    pub value: String,
}

pub(crate) trait CrystalMetadataAPIPostgreSQLStorage {
    async fn get_genesis_record_rows(
        &self,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<GenesisRecordRow>>;
}

impl CrystalMetadataAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_genesis_record_rows(
        &self,
        page: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<GenesisRecordRow>> {
        let offset = (page - 1) * page_size;
        let rows: Vec<GenesisRecordRow> =
            sqlx::query_as("SELECT id, key, value FROM genesis ORDER BY id ASC LIMIT $1 OFFSET $2")
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(&self.connection_pool)
                .await?;
        Ok(rows)
    }
}
