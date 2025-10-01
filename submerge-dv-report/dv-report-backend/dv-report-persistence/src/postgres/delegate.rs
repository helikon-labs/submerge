use crate::postgres::PostgreSQLStorage;
use dv_report_types::dv::delegate::{DelegateRow, DelegateTypeRow};

impl PostgreSQLStorage {
    pub async fn get_all_delegate_types(&self) -> anyhow::Result<Vec<DelegateTypeRow>> {
        let rows: Vec<DelegateTypeRow> = sqlx::query_as::<_, DelegateTypeRow>(
            "
            SELECT id, name, code
            FROM delegate_type
            ORDER BY name ASC
            ",
        )
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_delegate_by_id(&self, id: &str) -> anyhow::Result<Option<DelegateRow>> {
        let maybe_row: Option<DelegateRow> = sqlx::query_as::<_, DelegateRow>(
            r#"
            SELECT id, type_id, name, short_name, url, twitter
            FROM delegate
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.connection_pool)
        .await?;
        Ok(maybe_row)
    }

    pub async fn get_all_delegates(&self) -> anyhow::Result<Vec<DelegateRow>> {
        let rows: Vec<DelegateRow> = sqlx::query_as::<_, DelegateRow>(
            "
            SELECT id, type_id, name, short_name, url, twitter
            FROM delegate
            ORDER BY name ASC
            ",
        )
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows)
    }
}
