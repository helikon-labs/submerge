use sqlx::{Postgres, Transaction};
use std::str::FromStr;
use submerge_base::types::submerge::BlockTrace as SubmergeBlockTrace;
use submerge_base::types::submerge::BlockTraces;
use submerge_base::types::substrate::block::BlockHeader;
use submerge_base::types::substrate::block_trace::{
    BlockTrace as SubstrateBlockTrace, StorageMethod,
};
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_persistence::postgres::PostgreSQLStorage;

pub(crate) trait CrystalPostgreSQLStorage {
    async fn get_genesis_record_count(&self) -> anyhow::Result<u64>;
    async fn ingest_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()>;
    async fn get_next_block_number(&self, min: u64, max: u64) -> anyhow::Result<u64>;
    async fn save_trace_error(
        &self,
        block_hash: &str,
        block_number: u64,
        description: &str,
    ) -> anyhow::Result<()>;
    async fn delete_trace_error(
        &self,
        block_hash: &str,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    #[cfg(test)]
    async fn get_trace_error_count(&self) -> anyhow::Result<u32>;
    #[allow(clippy::too_many_arguments)]
    async fn ingest_block(
        &self,
        number: u64,
        hash: &str,
        header: &BlockHeader,
        timestamp: u64,
        is_finalized: bool,
        runtime_version: u32,
        extrinsic_count: u32,
        event_count: u32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    #[allow(clippy::too_many_arguments)]
    async fn ingest_block_trace(
        &self,
        number: u64,
        hash: &str,
        header: &BlockHeader,
        is_finalized: bool,
        runtime_version: u32,
        trace: &SubstrateBlockTrace,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()>;
    async fn get_block_traces_by_number(
        &self,
        block_number: u64,
    ) -> anyhow::Result<Vec<BlockTraces>>;
    async fn get_block_traces_by_hash(
        &self,
        block_hash: &[u8],
    ) -> anyhow::Result<Option<BlockTraces>>;
    async fn block_trace_exists(&self, block_hash: &str) -> anyhow::Result<bool>;

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
}

impl CrystalPostgreSQLStorage for PostgreSQLStorage {
    async fn get_genesis_record_count(&self) -> anyhow::Result<u64> {
        let record_count: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT key) FROM genesis")
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(record_count.0 as u64)
    }

    async fn ingest_genesis(&self, chainspec: &Chainspec) -> anyhow::Result<()> {
        log::info!("🔽 Processing genesis from chainspec file.");
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

    async fn get_next_block_number(&self, min: u64, max: u64) -> anyhow::Result<u64> {
        let row: (Option<i64>,) = sqlx::query_as(
            "SELECT MAX(block_number) FROM trace WHERE block_number >= $1 AND block_number <= $2",
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

    async fn save_trace_error(
        &self,
        block_hash: &str,
        block_number: u64,
        description: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO trace_error (block_hash, block_number, description)
            VALUES ($1, $2, $3)
            ON CONFLICT(block_hash) DO UPDATE
            SET description = EXCLUDED.description, created_at = now()
        "#,
        )
        .bind(hex::decode(block_hash)?)
        .bind(block_number as i64)
        .bind(description)
        .execute(&self.connection_pool)
        .await?;
        Ok(())
    }

    async fn delete_trace_error(
        &self,
        block_hash: &str,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM trace_error WHERE block_hash = $1")
            .bind(hex::decode(block_hash)?)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    #[cfg(test)]
    async fn get_trace_error_count(&self) -> anyhow::Result<u32> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT block_number) FROM trace_error")
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(row.0 as u32)
    }

    async fn ingest_block(
        &self,
        number: u64,
        hash: &str,
        header: &BlockHeader,
        timestamp: u64,
        is_finalized: bool,
        runtime_version: u32,
        extrinsic_count: u32,
        event_count: u32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let hash = hex::decode(hash)?;
        let parent_hash = hex::decode(&header.parent_hash)?;
        let state_root = hex::decode(&header.state_root)?;
        let extrinsic_root = hex::decode(&header.extrinsics_root)?;
        sqlx::query(
            r#"
                INSERT INTO block (hash, parent_hash, state_root, extrinsic_root, number, timestamp, runtime_version, is_finalized, extrinsic_count, event_count)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (hash) DO NOTHING
                "#,
        )
            .bind(&hash)
            .bind(&parent_hash)
            .bind(&state_root)
            .bind(&extrinsic_root)
            .bind(number as i64)
            .bind(timestamp as i64)
            .bind(runtime_version as i32)
            .bind(is_finalized)
            .bind(extrinsic_count as i32)
            .bind(event_count as i32)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    async fn ingest_block_trace(
        &self,
        number: u64,
        hash: &str,
        header: &BlockHeader,
        is_finalized: bool,
        runtime_version: u32,
        trace: &SubstrateBlockTrace,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let hash = hex::decode(hash)?;
        let parent_hash = hex::decode(&header.parent_hash)?;
        for (trace_index, event) in trace.events.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO trace (block_hash, block_parent_hash, block_number, runtime_version, is_finalized, trace_index, key, value, ext_id, method, parent_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                ON CONFLICT (block_hash, block_number, trace_index) DO NOTHING
                "#,
            )
                .bind(&hash)
                .bind(&parent_hash)
                .bind(number as i64)
                .bind(runtime_version as i32)
                .bind(is_finalized)
                .bind(trace_index as i32)
                .bind(&event.data_wrapper.data.key)
                .bind(&event.data_wrapper.data.value)
                .bind(&event.data_wrapper.data.ext_id)
                .bind(event.data_wrapper.data.method.to_string())
                .bind(&event.parent_id)
                .execute(&mut **tx)
                .await?;
        }
        Ok(())
    }

    async fn get_block_traces_by_number(
        &self,
        block_number: u64,
    ) -> anyhow::Result<Vec<BlockTraces>> {
        let block_hash_rows: Vec<(Vec<u8>,)> =
            sqlx::query_as("SELECT DISTINCT block_hash FROM trace WHERE block_number = $1")
                .bind(block_number as i64)
                .fetch_all(&self.connection_pool)
                .await?;
        let mut result = vec![];
        for block_hash_row in block_hash_rows.iter() {
            if let Some(block_traces) = self.get_block_traces_by_hash(&block_hash_row.0).await? {
                result.push(block_traces);
            }
        }
        Ok(result)
    }

    async fn get_block_traces_by_hash(
        &self,
        block_hash: &[u8],
    ) -> anyhow::Result<Option<BlockTraces>> {
        #[allow(clippy::type_complexity)]
        let rows: Vec<(Vec<u8>, i64, i32, bool, i32, String, String, String, String, Option<String>)> = sqlx::query_as("SELECT block_parent_hash, block_number, runtime_version, is_finalized, trace_index, key, value, ext_id, method, parent_id FROM trace WHERE block_hash = $1 ORDER BY trace_index ASC")
            .bind(block_hash)
            .fetch_all(&self.connection_pool)
            .await?;

        if let Some(first_row) = rows.first() {
            let block_hash_hex = format!("0x{}", hex::encode(block_hash));
            let block_parent_hash_hex = format!("0x{}", hex::encode(&first_row.0));
            let mut block_traces = BlockTraces {
                block_hash: block_hash_hex,
                block_parent_hash: block_parent_hash_hex,
                block_number: first_row.1 as u64,
                runtime_version: first_row.2 as u32,
                is_finalized: first_row.3,
                traces: vec![],
            };
            for row in rows.iter() {
                block_traces.traces.push(SubmergeBlockTrace {
                    index: row.4 as u32,
                    key: row.5.clone(),
                    value: row.6.clone(),
                    ext_id: row.7.clone(),
                    method: StorageMethod::from_str(&row.8)?,
                    parent_id: row.9.clone(),
                })
            }
            Ok(Some(block_traces))
        } else {
            Ok(None)
        }
    }

    async fn block_trace_exists(&self, block_hash: &str) -> anyhow::Result<bool> {
        let block_hash = hex::decode(block_hash)?;
        let record_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT trace_index)
            FROM trace
            WHERE block_hash = $1
            "#,
        )
        .bind(block_hash)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(record_count.0 > 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::persistence::{CrystalPostgreSQLStorage, PostgreSQLStorage};
    use std::fs;
    use submerge_base::types::substrate::chainspec::Chainspec;
    use submerge_substrate_client::SubstrateClient;

    async fn get_test_postgres() -> anyhow::Result<PostgreSQLStorage> {
        PostgreSQLStorage::new(
            "localhost",
            5432,
            "submerge",
            "submerge",
            "submerge_crystal_test",
            5,
            100,
        )
        .await
    }

    #[test_log::test(tokio::test)]
    async fn test_genesis_ingestion() -> Result<(), Box<dyn std::error::Error>> {
        let chainspec_path = "../_chainspecs/westend/sys/coretime-westend.json";
        let chainspec_json = fs::read_to_string(chainspec_path)?;
        let chainspec: Chainspec = serde_json::from_str(&chainspec_json)?;
        let postgres = get_test_postgres().await?;
        postgres.ingest_genesis(&chainspec).await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_ingest_blocks() -> Result<(), Box<dyn std::error::Error>> {
        let postgres = get_test_postgres().await?;
        let substrate_client = SubstrateClient::new(
            "https://rpc.helikon.io/coretime-westend-dev",
            "wss://rpc.helikon.io/coretime-westend-dev",
            30,
            30,
        )
        .await?;
        for number in 100..150 {
            let hash = substrate_client.get_block_hash(number).await?;
            let header = substrate_client.get_block_header(&hash).await?;
            let timestamp = substrate_client.get_block_timestamp(&hash).await?;
            let last_runtime_upgrade = substrate_client
                .get_last_runtime_upgrade_info(&hash)
                .await?;
            let mut tx = postgres.connection_pool.begin().await?;
            postgres
                .ingest_block(
                    number,
                    &hash,
                    &header,
                    timestamp,
                    true,
                    last_runtime_upgrade.spec_version,
                    0,
                    0,
                    &mut tx,
                )
                .await?;
            let trace = substrate_client.get_block_trace(&hash).await?;
            postgres
                .ingest_block_trace(
                    number,
                    &hash,
                    &header,
                    true,
                    last_runtime_upgrade.spec_version,
                    &trace,
                    &mut tx,
                )
                .await?;
            postgres.delete_trace_error(&hash, &mut tx).await?;
            tx.commit().await?;
        }
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_trace_error() -> Result<(), Box<dyn std::error::Error>> {
        let postgres = get_test_postgres().await?;
        let block_number = 100;
        let substrate_client = SubstrateClient::new(
            "https://rpc.helikon.io/coretime-westend-dev",
            "wss://rpc.helikon.io/coretime-westend-dev",
            30,
            30,
        )
        .await?;
        let block_hash = substrate_client.get_block_hash(block_number).await?;
        let mut tx = postgres.connection_pool.begin().await?;
        postgres.delete_trace_error(&block_hash, &mut tx).await?;
        tx.commit().await?;
        let pre_trace_error_count = postgres.get_trace_error_count().await?;
        postgres
            .save_trace_error(&block_hash, block_number, "error_description")
            .await?;
        let post_trace_error_count = postgres.get_trace_error_count().await?;
        assert_eq!(post_trace_error_count, pre_trace_error_count + 1);
        Ok(())
    }
}
