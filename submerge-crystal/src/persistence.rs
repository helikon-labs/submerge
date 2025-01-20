use sqlx::{Pool, Postgres};
use submerge_types::substrate::block_trace::BlockTrace;

pub(crate) async fn save_block_trace(
    pg_pool: &Pool<Postgres>,
    number: u64,
    trace: &BlockTrace,
) -> anyhow::Result<()> {
    let mut tx = pg_pool.begin().await?;
    for (trace_index, event) in trace.events.iter().enumerate() {
        let hash = hex::decode(&trace.block_hash)?;
        let parent_hash = hex::decode(&trace.parent_hash)?;
        sqlx::query(
            r#"
            INSERT INTO block_trace (hash, number, parent_hash, trace_index, key, value, value_encoded, ext_id, method, parent_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (hash, trace_index) DO NOTHING
            "#,
        )
        .bind(hash)
        .bind(number as i64)
        .bind(parent_hash)
        .bind(trace_index as i32)
        .bind(&event.data_wrapper.data.key)
        .bind(&event.data_wrapper.data.value)
        .bind(&event.data_wrapper.data.value_encoded)
        .bind(&event.data_wrapper.data.ext_id)
        .bind(event.data_wrapper.data.method.to_string())
        .bind(&event.parent_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
