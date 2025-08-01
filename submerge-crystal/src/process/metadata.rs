use crate::persistence::CrystalPostgreSQLStorage;
use crate::process::decode::ValueVisitor;
use crate::process::BlockProcessor;
use crate::types::metadata::util::get_metadata_version;
use crate::types::metadata::Metadata;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use lru::LruCache;
use parity_scale_codec::{Decode, Encode};
use serde_json::Value as JSONValue;
use std::num::NonZero;
use std::sync::{Arc, LazyLock};
use submerge_util::serde::strip_nuls;
use tokio::sync::RwLock;

const METADATA_CACHE_SIZE: usize = 10;

static METADATA_CACHE: LazyLock<RwLock<LruCache<u32, Arc<RuntimeMetadata>>>> =
    LazyLock::new(|| RwLock::new(LruCache::new(NonZero::new(METADATA_CACHE_SIZE).unwrap())));

impl BlockProcessor {
    pub async fn get_metadata(
        &self,
        block_hash: &[u8],
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
                        .get_metadata_hex_string_at_block(&hex::encode(block_hash))
                        .await?;
                    let mut metadata_bytes: &[u8] = &hex::decode(metadata_hex_string)?;
                    let metadata_prefixed = RuntimeMetadataPrefixed::decode(&mut metadata_bytes)?;
                    let metadata_json = serde_json::to_value(&metadata_prefixed)?;
                    let mut metadata: Metadata = (&metadata_prefixed.1).try_into()?;
                    let metadata_version = get_metadata_version(&metadata_prefixed.1);
                    for pallet in metadata.pallets.iter_mut() {
                        for constant in pallet.constants.iter_mut() {
                            let mut bytes: &[u8] = &constant.value;
                            if metadata_version < 14 {
                                let legacy_decode_api_client = if let Some(client) =
                                    &self.legacy_decode_api_client
                                {
                                    client
                                } else {
                                    anyhow::bail!("Legacy decode API client is not set. legacy_decode_api_url parameter not set.");
                                };
                                constant.value_json = Some(
                                    legacy_decode_api_client
                                        .decode_type(
                                            block_hash,
                                            spec_version,
                                            &constant.type_name,
                                            bytes,
                                        )
                                        .await?,
                                );
                            } else {
                                let type_id = constant.type_id.ok_or(anyhow::Error::msg(
                                    "Type id not found in constant with metadata version ≥ 14.",
                                ))?;
                                match &metadata_prefixed.1 {
                                    RuntimeMetadata::V14(metadata_v14) => {
                                        let visitor = ValueVisitor::new(0, None);
                                        let mut value: JSONValue =
                                            scale_decode::visitor::decode_with_visitor(
                                                &mut bytes,
                                                type_id,
                                                &metadata_v14.types,
                                                visitor,
                                            )?
                                            .into();
                                        strip_nuls(&mut value);
                                        constant.value_json = Some(value);
                                    }
                                    _ => anyhow::bail!(
                                        "Unsupported metadata version {metadata_version}."
                                    ),
                                }
                            }
                        }
                    }
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
