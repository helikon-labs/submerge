use serde_json::Value as JSONValue;
use sqlx::FromRow;
use submerge_persistence::postgres::PostgreSQLStorage;

use crate::types::api::dto::metadata::{
    Metadata, MetadataPallet, MetadataPalletCall, MetadataPalletConstant, MetadataPalletError,
    MetadataPalletEvent, MetadataPalletStorageItem,
};

#[derive(Clone, Debug, FromRow)]
pub struct PalletConstantRow {
    pub index: i32,
    pub name: String,
    pub type_id: Option<i32>,
    pub type_name: String,
    pub value: Vec<u8>,
    pub value_json: Option<JSONValue>,
    pub docs: Vec<String>,
}

pub(crate) trait CrystalMetadataAPIPostgreSQLStorage {
    async fn get_metadata_count(&self) -> anyhow::Result<u64>;
    async fn get_metadata_list(
        &self,
        page_number: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<Metadata>>;
    async fn metadata_exists(&self, spec_version: u32) -> anyhow::Result<bool>;
    async fn get_metadata_json(&self, spec_version: u32) -> anyhow::Result<Option<JSONValue>>;
    async fn get_metadata_bytes(&self, spec_version: u32) -> anyhow::Result<Option<Vec<u8>>>;
    async fn get_metadata_pallets(&self, spec_version: u32) -> anyhow::Result<Vec<MetadataPallet>>;
    async fn metadata_pallet_exists(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<bool>;
    async fn get_metadata_pallet_calls(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<Vec<MetadataPalletCall>>;
    async fn get_metadata_pallet_constants(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<Vec<MetadataPalletConstant>>;
    async fn get_metadata_pallet_errors(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<Vec<MetadataPalletError>>;
    async fn get_metadata_pallet_events(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<Vec<MetadataPalletEvent>>;
    async fn get_metadata_pallet_storage_items(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<Vec<MetadataPalletStorageItem>>;
}

impl CrystalMetadataAPIPostgreSQLStorage for PostgreSQLStorage {
    async fn get_metadata_count(&self) -> anyhow::Result<u64> {
        let record_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM metadata")
            .fetch_one(&self.connection_pool)
            .await?;
        Ok(record_count.0 as u64)
    }
    async fn get_metadata_list(
        &self,
        page_number: u64,
        page_size: u64,
    ) -> anyhow::Result<Vec<Metadata>> {
        let offset = (page_number - 1) * page_size;
        let rows: Vec<(i32, i32)> = sqlx::query_as(
            "SELECT spec_version, metadata_version FROM metadata ORDER BY spec_version ASC LIMIT $1 OFFSET $2",
        )
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows
            .iter()
            .map(|row| Metadata {
                spec_version: row.0 as u32,
                metadata_version: row.1 as u32,
            })
            .collect())
    }

    async fn metadata_exists(&self, spec_version: u32) -> anyhow::Result<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT(spec_version)) FROM metadata
            WHERE spec_version = $1
            "#,
        )
        .bind(spec_version as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count.0 > 0)
    }

    async fn get_metadata_json(&self, spec_version: u32) -> anyhow::Result<Option<JSONValue>> {
        let row: Option<(JSONValue,)> =
            sqlx::query_as("SELECT metadata_json FROM metadata WHERE spec_version = $1")
                .bind(spec_version as i32)
                .fetch_optional(&self.connection_pool)
                .await?;
        Ok(row.map(|row| row.0))
    }

    async fn get_metadata_bytes(&self, spec_version: u32) -> anyhow::Result<Option<Vec<u8>>> {
        let row: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT metadata_bytes FROM metadata WHERE spec_version = $1")
                .bind(spec_version as i32)
                .fetch_optional(&self.connection_pool)
                .await?;
        Ok(row.map(|row| row.0))
    }

    async fn get_metadata_pallets(&self, spec_version: u32) -> anyhow::Result<Vec<MetadataPallet>> {
        let rows: Vec<(i32, String)> = sqlx::query_as(
            r#"
            SELECT index, name FROM metadata_pallet
            WHERE spec_version = $1
            ORDER BY index ASC
            "#,
        )
        .bind(spec_version as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows
            .iter()
            .map(|row| MetadataPallet {
                index: row.0 as u32,
                name: row.1.clone(),
            })
            .collect())
    }

    async fn metadata_pallet_exists(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT((spec_version, index))) FROM metadata_pallet
            WHERE spec_version = $1 AND index = $2
            "#,
        )
        .bind(spec_version as i32)
        .bind(pallet_index as i32)
        .fetch_one(&self.connection_pool)
        .await?;
        Ok(count.0 > 0)
    }

    async fn get_metadata_pallet_calls(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<Vec<MetadataPalletCall>> {
        let rows: Vec<(i32, String, Vec<String>)> = sqlx::query_as(
            r#"
            SELECT index, name, docs FROM metadata_pallet_call
            WHERE spec_version = $1 AND pallet_index = $2
            ORDER BY index ASC
            "#,
        )
        .bind(spec_version as i32)
        .bind(pallet_index as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows
            .iter()
            .map(|row| MetadataPalletCall {
                index: row.0 as u32,
                name: row.1.clone(),
                docs: row.2.clone(),
            })
            .collect())
    }

    async fn get_metadata_pallet_constants(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<Vec<MetadataPalletConstant>> {
        let rows: Vec<PalletConstantRow> = sqlx::query_as(
        r#"
            SELECT index, name, type_id, type_name, value, value_json, docs FROM metadata_pallet_constant
            WHERE spec_version = $1 AND pallet_index = $2
            ORDER BY index ASC
            "#,
        )
        .bind(spec_version as i32)
        .bind(pallet_index as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows
            .iter()
            .map(|row| MetadataPalletConstant {
                index: row.index as u32,
                name: row.name.clone(),
                type_id: row.type_id.map(|id| id as u32),
                type_name: row.type_name.clone(),
                value_hex: format!("0x{}", hex::encode(&row.value)),
                value: row.value_json.clone(),
                docs: row.docs.clone(),
            })
            .collect())
    }

    async fn get_metadata_pallet_errors(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<Vec<MetadataPalletError>> {
        let rows: Vec<(i32, String, Vec<String>)> = sqlx::query_as(
            r#"
            SELECT index, name, docs FROM metadata_pallet_error
            WHERE spec_version = $1 AND pallet_index = $2
            ORDER BY index ASC
            "#,
        )
        .bind(spec_version as i32)
        .bind(pallet_index as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows
            .iter()
            .map(|row| MetadataPalletError {
                index: row.0 as u32,
                name: row.1.clone(),
                docs: row.2.clone(),
            })
            .collect())
    }

    async fn get_metadata_pallet_events(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<Vec<MetadataPalletEvent>> {
        let rows: Vec<(i32, String, Vec<String>)> = sqlx::query_as(
            r#"
            SELECT index, name, docs FROM metadata_pallet_event
            WHERE spec_version = $1 AND pallet_index = $2
            ORDER BY index ASC
            "#,
        )
        .bind(spec_version as i32)
        .bind(pallet_index as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows
            .iter()
            .map(|row| MetadataPalletEvent {
                index: row.0 as u32,
                name: row.1.clone(),
                docs: row.2.clone(),
            })
            .collect())
    }

    async fn get_metadata_pallet_storage_items(
        &self,
        spec_version: u32,
        pallet_index: u32,
    ) -> anyhow::Result<Vec<MetadataPalletStorageItem>> {
        let rows: Vec<(i32, String, String, Vec<String>)> = sqlx::query_as(
            r#"
            SELECT index, name, key, docs FROM metadata_pallet_storage_item
            WHERE spec_version = $1 AND pallet_index = $2
            ORDER BY index ASC
            "#,
        )
        .bind(spec_version as i32)
        .bind(pallet_index as i32)
        .fetch_all(&self.connection_pool)
        .await?;
        Ok(rows
            .iter()
            .map(|row| MetadataPalletStorageItem {
                index: row.0 as u32,
                name: row.1.clone(),
                key: if row.2.starts_with("0x") {
                    row.2.clone()
                } else {
                    format!("0x{}", row.2)
                },
                docs: row.3.clone(),
            })
            .collect())
    }
}
