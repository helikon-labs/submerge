use crate::persistence::CrystalPostgreSQLStorage;
use crate::process::BlockProcessor;
use crate::types::metadata::Metadata;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use lru::LruCache;
use parity_scale_codec::{Decode, Encode};
use std::num::NonZero;
use std::sync::{Arc, LazyLock};
use tokio::sync::RwLock;

const METADATA_CACHE_SIZE: usize = 10;

static METADATA_CACHE: LazyLock<RwLock<LruCache<u32, Arc<RuntimeMetadata>>>> =
    LazyLock::new(|| RwLock::new(LruCache::new(NonZero::new(METADATA_CACHE_SIZE).unwrap())));

pub fn get_metadata_version(metadata: &RuntimeMetadata) -> u32 {
    match &metadata {
        RuntimeMetadata::V8(_) => 8,
        RuntimeMetadata::V9(_) => 9,
        RuntimeMetadata::V10(_) => 10,
        RuntimeMetadata::V11(_) => 11,
        RuntimeMetadata::V12(_) => 12,
        RuntimeMetadata::V13(_) => 13,
        RuntimeMetadata::V14(_) => 14,
        RuntimeMetadata::V15(_) => 15,
        RuntimeMetadata::V16(_) => 16,
        _ => unimplemented!("Unsupported metadata version."),
    }
}

impl BlockProcessor {
    pub async fn get_metadata(
        &self,
        block_hash_hex: &str,
        spec_version: u32,
    ) -> anyhow::Result<Arc<RuntimeMetadata>> {
        let mut metadata_cache = METADATA_CACHE.write().await;
        let metadata = {
            if !metadata_cache.contains(&spec_version) {
                let metadata = if let Some(db_metadata) =
                    self.postgres.get_metadata(spec_version).await?
                {
                    db_metadata.1
                } else {
                    let metadata_hex_string = self
                        .substrate_client
                        .get_metadata_hex_string_at_block(block_hash_hex)
                        .await?;
                    let mut metadata_bytes: &[u8] = &hex::decode(metadata_hex_string)?;
                    let metadata_prefixed = RuntimeMetadataPrefixed::decode(&mut metadata_bytes)?;
                    let metadata_json = serde_json::to_value(&metadata_prefixed)?;
                    let metadata: Metadata = (&metadata_prefixed.1).try_into()?;
                    self.postgres
                        .ingest_metadata(
                            spec_version,
                            get_metadata_version(&metadata_prefixed.1),
                            &metadata_prefixed.encode(),
                            &metadata_json,
                            &metadata,
                        )
                        .await?;
                    metadata_prefixed.1
                };
                metadata_cache.put(spec_version, Arc::new(metadata));
            }
            metadata_cache.get(&spec_version).unwrap()
        };
        Ok(metadata.clone())
    }
}
