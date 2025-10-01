use crate::postgres::PostgreSQLStorage;
use dv_report_types::dv::delegation::DelegationRow;

impl PostgreSQLStorage {
    pub async fn get_delegations_for_delegate(
        &self,
        delegate_id: &str,
    ) -> anyhow::Result<Vec<DelegationRow>> {
        let rows: Vec<DelegationRow> = sqlx::query_as::<_, DelegationRow>(
            r#"
            SELECT id, cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id, start_block_hash, start_extrinsic_hash, start_extrinsic_index, end_block_hash, end_extrinsic_hash, end_extrinsic_index
            FROM delegation
            WHERE delegate_id = $1
            "#,
        )
            .bind(delegate_id)
            .fetch_all(&self.connection_pool)
            .await?;
        Ok(rows)
    }

    pub async fn get_network_cohort_delegation_for_delegate(
        &self,
        network_id: u32,
        cohort_number: u32,
        delegate_id: &str,
    ) -> anyhow::Result<DelegationRow> {
        let row: DelegationRow = sqlx::query_as::<_, DelegationRow>(
            r#"
            SELECT id, cohort_number, network_id, delegator_account_id, delegate_id, delegate_account_id, start_block_hash, start_extrinsic_hash, start_extrinsic_index, end_block_hash, end_extrinsic_hash, end_extrinsic_index
            FROM delegation
            WHERE network_id = $1 AND cohort_number = $2 AND  delegate_id = $3
            "#,
        )
            .bind(network_id as i32)
            .bind(cohort_number as i32)
            .bind(delegate_id)
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(row)
    }
}
