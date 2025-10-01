use crate::postgres::PostgreSQLStorage;
use dv_report_types::substrate::block::{Block, BlockRow};
use sqlx::{Postgres, Transaction};

impl PostgreSQLStorage {
    pub async fn save_block(
        &self,
        network_id: u32,
        block: &Block,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO block (network_id, hash, number, timestamp, parent_hash)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (network_id, hash) DO NOTHING
            "#,
        )
        .bind(network_id as i32)
        .bind(block.hash.as_str())
        .bind(block.number as i64)
        .bind(block.timestamp as i64)
        .bind(block.parent_hash.as_str())
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn get_max_block_number(&self, network_id: u32) -> anyhow::Result<i64> {
        let row: (i64,) =
            sqlx::query_as("SELECT COALESCE(MAX(number), -1) FROM block WHERE network_id = $1")
                .bind(network_id as i32)
                .fetch_one(&self.connection_pool)
                .await?;
        Ok(row.0)
    }

    pub async fn get_block(&self, network_id: u32, hash: &str) -> anyhow::Result<Block> {
        let row: BlockRow = sqlx::query_as::<_, BlockRow>(
            r#"
            SELECT network_id, hash, number, timestamp, parent_hash FROM BLOCK
            WHERE network_id = $1 and hash = $2
            "#,
        )
        .bind(network_id as i32)
        .bind(hash)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(row.into())
    }
}
