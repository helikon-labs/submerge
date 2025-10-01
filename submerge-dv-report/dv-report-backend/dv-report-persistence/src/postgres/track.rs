use crate::postgres::PostgreSQLStorage;
use dv_report_types::substrate::track::TrackRow;

impl PostgreSQLStorage {
    pub async fn get_all_tracks_for_network(
        &self,
        network_id: u32,
    ) -> anyhow::Result<Vec<TrackRow>> {
        let rows: Vec<TrackRow> = sqlx::query_as::<_, TrackRow>(
            r#"
            SELECT network_id, id, name
            FROM track
            WHERE network_id = $1
            ORDER BY id ASC
            "#,
        )
        .bind(network_id as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_all_tracks_for_network_cohort(
        &self,
        network_id: u32,
        cohort_number: u32,
    ) -> anyhow::Result<Vec<TrackRow>> {
        let rows: Vec<TrackRow> = sqlx::query_as::<_, TrackRow>(
            r#"
            SELECT network_id, id, name
            FROM track
            WHERE id IN (
                SELECT track_id FROM cohort_track
                WHERE network_id = $1 AND cohort_number = $2
            )
            AND network_id = $1
            ORDER BY id ASC
            "#,
        )
        .bind(network_id as i32)
        .bind(cohort_number as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }
}
