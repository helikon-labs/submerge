use crate::postgres::PostgreSQLStorage;
use dv_report_types::substrate::account_id::AccountId;
use dv_report_types::substrate::vote::Tally;
use sqlx::{Postgres, Transaction};

impl PostgreSQLStorage {
    pub async fn save_referendum_submitted_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        track_id: u32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_submitted (network_id, block_hash, referendum_index, track_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(network_id, block_hash, referendum_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(block_hash)
            .bind(referendum_index as i32)
            .bind(track_id as i32)
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_decision_deposit_placed_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        amount: u128,
        who: &AccountId,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_decision_deposit_placed (network_id, block_hash, referendum_index, amount, who)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(network_id, block_hash, referendum_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(block_hash)
            .bind(referendum_index as i32)
            .bind(amount.to_string())
            .bind(who.to_string())
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_decision_deposit_refunded_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        amount: u128,
        who: &AccountId,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_decision_deposit_refunded (network_id, block_hash, referendum_index, amount, who)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(network_id, block_hash, referendum_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(block_hash)
            .bind(referendum_index as i32)
            .bind(amount.to_string())
            .bind(who.to_string())
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_deposit_slashed_event(
        &self,
        network_id: u32,
        block_hash: &str,
        amount: u128,
        who: &AccountId,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_deposit_slashed (network_id, block_hash, amount, who)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(network_id as i32)
        .bind(block_hash)
        .bind(amount.to_string())
        .bind(who.to_string())
        .fetch_one(&mut **tx)
        .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_decision_started_event(
        &self,
        network_id: u32,
        block_hash: &str,
        track_id: u16,
        referendum_index: u32,
        tally: &Tally,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_decision_started (network_id, block_hash, track_id, referendum_index, ayes, nays, support)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(block_hash)
            .bind(track_id as i32)
            .bind(referendum_index as i32)
            .bind(tally.ayes.to_string())
            .bind(tally.nays.to_string())
            .bind(tally.support.to_string())
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_confirm_started_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_confirm_started (network_id, block_hash, referendum_index)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(network_id as i32)
        .bind(block_hash)
        .bind(referendum_index as i32)
        .fetch_one(&mut **tx)
        .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_confirm_aborted_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_confirm_aborted (network_id, block_hash, referendum_index)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(network_id as i32)
        .bind(block_hash)
        .bind(referendum_index as i32)
        .fetch_one(&mut **tx)
        .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_confirmed_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        tally: &Tally,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_confirmed (network_id, block_hash, referendum_index, ayes, nays, support)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(block_hash)
            .bind(referendum_index as i32)
            .bind(tally.ayes.to_string())
            .bind(tally.nays.to_string())
            .bind(tally.support.to_string())
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_approved_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_approved (network_id, block_hash, referendum_index)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(network_id as i32)
        .bind(block_hash)
        .bind(referendum_index as i32)
        .fetch_one(&mut **tx)
        .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_rejected_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        tally: &Tally,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_rejected (network_id, block_hash, referendum_index, ayes, nays, support)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(block_hash)
            .bind(referendum_index as i32)
            .bind(tally.ayes.to_string())
            .bind(tally.nays.to_string())
            .bind(tally.support.to_string())
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_cancelled_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        tally: &Tally,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_cancelled (network_id, block_hash, referendum_index, ayes, nays, support)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(block_hash)
            .bind(referendum_index as i32)
            .bind(tally.ayes.to_string())
            .bind(tally.nays.to_string())
            .bind(tally.support.to_string())
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_timed_out_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        tally: &Tally,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_timed_out (network_id, block_hash, referendum_index, ayes, nays, support)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(block_hash)
            .bind(referendum_index as i32)
            .bind(tally.ayes.to_string())
            .bind(tally.nays.to_string())
            .bind(tally.support.to_string())
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_killed_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        tally: &Tally,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_killed (network_id, block_hash, referendum_index, ayes, nays, support)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(block_hash)
            .bind(referendum_index as i32)
            .bind(tally.ayes.to_string())
            .bind(tally.nays.to_string())
            .bind(tally.support.to_string())
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }

    pub async fn save_referendum_submission_deposit_refunded_event(
        &self,
        network_id: u32,
        block_hash: &str,
        referendum_index: u32,
        amount: u128,
        who: &AccountId,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<i32> {
        let result: (i32,) = sqlx::query_as(
            r#"
            INSERT INTO referendum_event_submission_deposit_refunded (network_id, block_hash, referendum_index, amount, who)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(network_id, block_hash, referendum_index) DO NOTHING
            RETURNING id
            "#,
        )
            .bind(network_id as i32)
            .bind(block_hash)
            .bind(referendum_index as i32)
            .bind(amount.to_string())
            .bind(who.to_string())
            .fetch_one(&mut **tx)
            .await?;
        Ok(result.0)
    }
}
