use crate::api::legacy::LegacyDecodeAPIClient;
use crate::types::metadata::Metadata;
use frame_metadata::RuntimeMetadata;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, LazyLock};
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::SubstrateClient;
use submerge_util::serde::strip_nuls;
use tokio::sync::RwLock;

use crate::persistence::CrystalPostgreSQLStorage;
use crate::types::{decode::ValueVisitor, metadata::util::get_metadata_version};
use frame_metadata::RuntimeMetadataPrefixed;
use parity_scale_codec::{Decode, Encode};
use serde_json::Value as JSONValue;

const METADATA_CACHE_SIZE: NonZeroUsize =
    NonZeroUsize::new(10).expect("Metadata cache size is non-zero");

static PARSED_METADATA_CACHE: LazyLock<RwLock<LruCache<u32, Arc<Metadata>>>> =
    LazyLock::new(|| RwLock::new(LruCache::new(METADATA_CACHE_SIZE)));
static METADATA_CACHE: LazyLock<RwLock<LruCache<u32, Arc<RuntimeMetadata>>>> =
    LazyLock::new(|| RwLock::new(LruCache::new(METADATA_CACHE_SIZE)));

pub async fn get_parsed_metadata(
    block_hash: &[u8],
    spec_version: u32,
    postgres: &PostgreSQLStorage,
    substrate_client: &SubstrateClient,
    legacy_decode_api_client: &Option<LegacyDecodeAPIClient>,
) -> anyhow::Result<Arc<Metadata>> {
    {
        let cache = PARSED_METADATA_CACHE.read().await;
        if let Some(metadata) = cache.peek(&spec_version) {
            return Ok(metadata.clone());
        }
    }
    let metadata = get_metadata(
        block_hash,
        spec_version,
        postgres,
        substrate_client,
        legacy_decode_api_client,
    )
    .await?;
    let mut parsed_metadata: Metadata = (&*metadata).try_into()?;
    // initialize database ids for metadata items
    for pallet in parsed_metadata.pallets.iter_mut() {
        pallet.id = postgres
            .get_metadata_pallet_id(spec_version, pallet.index)
            .await?;
        for event in pallet.events.iter_mut() {
            event.id = postgres
                .get_metadata_event_id(pallet.id, event.index)
                .await?;
        }
        for constant in pallet.constants.iter_mut() {
            constant.id = postgres
                .get_metadata_constant_id(pallet.id, constant.index)
                .await?;
        }
        for call in pallet.calls.iter_mut() {
            call.id = postgres.get_metadata_call_id(pallet.id, call.index).await?;
        }
        for storage_item in pallet.storage_items.iter_mut() {
            storage_item.id = postgres
                .get_metadata_storage_item_id(pallet.id, storage_item.index)
                .await?;
        }
        for error in pallet.errors.iter_mut() {
            error.id = postgres
                .get_metadata_error_id(pallet.id, error.index)
                .await?;
        }
    }
    let parsed_metadata_arc = Arc::new(parsed_metadata);
    {
        let mut cache = PARSED_METADATA_CACHE.write().await;
        cache.put(spec_version, parsed_metadata_arc.clone());
    }
    Ok(parsed_metadata_arc)
}

pub async fn get_metadata(
    block_hash: &[u8],
    spec_version: u32,
    postgres: &PostgreSQLStorage,
    substrate_client: &SubstrateClient,
    legacy_decode_api_client: &Option<LegacyDecodeAPIClient>,
) -> anyhow::Result<Arc<RuntimeMetadata>> {
    {
        let cache = METADATA_CACHE.read().await;
        if let Some(metadata) = cache.peek(&spec_version) {
            return Ok(metadata.clone());
        }
    }
    let metadata = if let Some(db_metadata) = postgres.get_metadata(spec_version).await? {
        db_metadata.1
    } else {
        let metadata_hex_string = substrate_client
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
                    let legacy_decode_api_client = if let Some(client) = legacy_decode_api_client {
                        client
                    } else {
                        anyhow::bail!("Legacy decode API client is not set. legacy_decode_api_url parameter not set.");
                    };
                    constant.value_json = Some(
                        legacy_decode_api_client
                            .decode_type(block_hash, spec_version, &constant.type_name, bytes)
                            .await?,
                    );
                } else {
                    let type_id = constant.type_id.ok_or(anyhow::Error::msg(
                        "Type id not found in constant with metadata version ≥ 14.",
                    ))?;
                    match &metadata_prefixed.1 {
                        RuntimeMetadata::V14(metadata_v14) => {
                            let visitor = ValueVisitor::new(0, None);
                            let mut value: JSONValue = scale_decode::visitor::decode_with_visitor(
                                &mut bytes,
                                type_id,
                                &metadata_v14.types,
                                visitor,
                            )?
                            .into();
                            strip_nuls(&mut value);
                            constant.value_json = Some(value);
                        }
                        RuntimeMetadata::V15(metadata_v15) => {
                            let visitor = ValueVisitor::new(0, None);
                            let mut value: JSONValue = scale_decode::visitor::decode_with_visitor(
                                &mut bytes,
                                type_id,
                                &metadata_v15.types,
                                visitor,
                            )?
                            .into();
                            strip_nuls(&mut value);
                            constant.value_json = Some(value);
                        }
                        _ => anyhow::bail!("Unsupported metadata version {metadata_version}."),
                    }
                }
            }
        }
        postgres
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
    let metadata_arc = Arc::new(metadata);
    {
        let mut cache = METADATA_CACHE.write().await;
        cache.put(spec_version, metadata_arc.clone());
    }
    Ok(metadata_arc)
}
