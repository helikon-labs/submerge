use crate::postgres::PostgreSQLStorage;
use dv_report_types::substrate::network::{Network, NetworkRow};

impl PostgreSQLStorage {
    pub async fn get_all_networks(&self) -> anyhow::Result<Vec<Network>> {
        let rows: Vec<NetworkRow> = sqlx::query_as::<_, NetworkRow>(
            r#"
            SELECT id, hash, chain, display, ss58_prefix, token_ticker, token_decimal_count
            FROM network
            ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows
            .iter()
            .map(|row| Network::from_id(row.id as u32))
            .collect())
    }
}
