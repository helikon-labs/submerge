use sqlx::{Pool, Postgres, Transaction};
use std::fs;
use std::path::Path;
use submerge_persistence::postgres::new_postgres_connection_pool;
use submerge_types::substrate::block_trace::BlockTrace;
use submerge_types::substrate::chainspec::Chainspec;

pub struct PostgreSQLStorage {
    connection_pool: Pool<Postgres>,
}

impl PostgreSQLStorage {
    pub async fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        database_name: &str,
        connection_timeout_secs: u64,
        pool_max_connections: u32,
    ) -> anyhow::Result<PostgreSQLStorage> {
        Ok(PostgreSQLStorage {
            connection_pool: new_postgres_connection_pool(
                host,
                port,
                username,
                password,
                database_name,
                connection_timeout_secs,
                pool_max_connections,
            )
            .await?,
        })
    }
}

impl PostgreSQLStorage {
    pub async fn get_genesis_record_count(&self) -> anyhow::Result<u64> {
        let record_count: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT key) FROM genesis")
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(record_count.0 as u64)
    }

    pub(crate) async fn ingest_genesis(
        &self,
        chainspec_path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        log::info!(
            "🔽 Processing genesis from chainspec file: {:?}",
            chainspec_path.as_ref(),
        );
        let chainspec_json = fs::read_to_string(&chainspec_path)?;
        let chainspec: Chainspec = serde_json::from_str(&chainspec_json)?;
        if self.get_genesis_record_count().await? > 0 {
            log::info!("🔁 Genesis had already been processed.");
            return Ok(());
        }
        let mut tx = self.connection_pool.begin().await?;
        for (key, value) in chainspec.genesis.raw.top.iter() {
            Self::ingest_genesis_item(&mut tx, key, value).await?;
        }
        tx.commit().await?;
        log::info!(
            "✅ Processed {} storage items from the chainspec file.",
            chainspec.genesis.raw.top.len()
        );
        Ok(())
    }

    async fn ingest_genesis_item(
        tx: &mut Transaction<'_, Postgres>,
        key: &str,
        value: &str,
    ) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO genesis (key, value) VALUES ($1, $2) ON CONFLICT(key) DO NOTHING")
            .bind(key)
            .bind(value)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    pub(crate) async fn get_next_block_number(&self, min: u64, max: u64) -> anyhow::Result<u64> {
        let row: (Option<i64>,) = sqlx::query_as(
            "SELECT MAX(number) FROM block_trace WHERE number >= $1 AND number <= $2",
        )
        .bind(min as i64)
        .bind(max as i64)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(if let Some(min_in_range) = row.0 {
            min_in_range as u64 + 1
        } else {
            min
        })
    }

    pub(crate) async fn block_trace_exists(&self, hash: &str) -> anyhow::Result<bool> {
        let hash = hex::decode(hash)?;
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT trace_index)
            FROM block_trace
            WHERE hash = $1
            "#,
        )
        .bind(hash)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }

    pub(crate) async fn ingest_block_trace(
        &self,
        number: u64,
        trace: &BlockTrace,
    ) -> anyhow::Result<()> {
        let mut tx = self.connection_pool.begin().await?;
        for (trace_index, event) in trace.events.iter().enumerate() {
            let hash = hex::decode(&trace.block_hash)?;
            let parent_hash = hex::decode(&trace.parent_hash)?;
            let key = hex::decode(&event.data_wrapper.data.key)?;
            sqlx::query(
                r#"
                INSERT INTO block_trace (hash, number, parent_hash, trace_index, key, value, ext_id, method, parent_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (hash, trace_index) DO NOTHING
                "#,
            )
                .bind(hash)
                .bind(number as i64)
                .bind(parent_hash)
                .bind(trace_index as i32)
                .bind(key)
                .bind(&event.data_wrapper.data.value)
                .bind(&event.data_wrapper.data.ext_id)
                .bind(event.data_wrapper.data.method.to_string())
                .bind(&event.parent_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::persistence::PostgreSQLStorage;
    use submerge_substrate_client::SubstrateClient;

    #[test_log::test(tokio::test)]
    async fn test_genesis_ingestion() -> Result<(), Box<dyn std::error::Error>> {
        let postgres = PostgreSQLStorage::new(
            "localhost",
            5432,
            "submerge",
            "submerge",
            "submerge_crystal_test",
            5,
            100,
        )
        .await?;
        let chainspecs_path = "../_chainspecs/coretime-westend.json";
        postgres.ingest_genesis(chainspecs_path).await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_substrate_rpc_url() -> Result<(), Box<dyn std::error::Error>> {
        let postgres = PostgreSQLStorage::new(
            "localhost",
            5432,
            "submerge",
            "submerge",
            "submerge_crystal_test",
            5,
            100,
        )
            .await?;
        let substrate_client =
            SubstrateClient::new("wss://rpc.helikon.io/coretime-westend-dev", 30, 30).await?;
        for number in 100..120 {
            let hash = substrate_client.get_block_hash(number).await?;
            let trace = substrate_client.get_block_trace(&hash).await?;
            postgres.ingest_block_trace(number, &trace).await?;
        }
        Ok(())
    }
}
