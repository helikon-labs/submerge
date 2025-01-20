use sqlx::{Pool, Postgres};
use std::str::FromStr;
use submerge_types::substrate::block_trace::{BlockTraceData, StorageMethod};

pub(crate) async fn get_block_traces(
    pg_pool: &Pool<Postgres>,
    block_hash: &str,
) -> anyhow::Result<Vec<BlockTraceData>> {
    let block_hash = hex::decode(block_hash)?;
    let db_block_traces: Vec<(
        Vec<u8>,
        Vec<u8>,
        i32,
        String,
        String,
        Option<String>,
        String,
        String,
    )> = sqlx::query_as(
        r#"
            SELECT hash, parent_hash, trace_index, key, value, value_encoded, ext_id, method
            FROM block_trace
            WHERE hash = $1
            ORDER BY trace_index ASC
            "#,
    )
    .bind(&block_hash)
    .fetch_all(pg_pool)
    .await?;
    let mut block_traces = Vec::new();
    for (i, db_block_trace) in db_block_traces.iter().enumerate() {
        block_traces.push(BlockTraceData {
            key: db_block_trace.3.clone(),
            value: db_block_trace.4.clone(),
            value_encoded: db_block_trace.5.clone(),
            ext_id: db_block_trace.6.clone(),
            method: StorageMethod::from_str(&db_block_trace.7)?,
        });
    }
    Ok(block_traces)
}
