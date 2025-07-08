use std::num::NonZero;

use crate::decode::{get_metadata_version, JsonValueVisitor};
use crate::persistence::CrystalPostgreSQLStorage;
use frame_metadata::v16::StorageHasher;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use lru::LruCache;
use parity_scale_codec::{Compact, Decode, Encode, Input};
use serde_json::Value as JsonValue;
use sqlx::{Postgres, Transaction};
use std::sync::{Arc, LazyLock};
use submerge_base::args::{PostgreSQLArgs, RPCArgs};
use submerge_base::types::substrate::block::BlockHeader;
use submerge_base::types::substrate::block_trace::{BlockTrace, StorageMethod};
use submerge_base::types::substrate::{Balance, MultiAddress, Signature};
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::SubstrateClient;
use submerge_util::substrate::storage::{self, get_storage_plain_key};
use tokio::sync::RwLock;

use crate::legacy::LegacyDecodeAPIClient;

const METADATA_CACHE_SIZE: usize = 10;

static METADATA_CACHE: LazyLock<RwLock<LruCache<u32, Arc<RuntimeMetadata>>>> =
    LazyLock::new(|| RwLock::new(LruCache::new(NonZero::new(METADATA_CACHE_SIZE).unwrap())));

pub struct BlockProcessor {
    postgres: PostgreSQLStorage,
    substrate_client: SubstrateClient,
    legacy_decode_api_client: LegacyDecodeAPIClient,
}

impl BlockProcessor {
    pub async fn new(postgres_args: &PostgreSQLArgs, rpc_args: &RPCArgs) -> anyhow::Result<Self> {
        let postgres = PostgreSQLStorage::new(postgres_args).await?;
        let substrate_client = SubstrateClient::new(rpc_args).await?;
        let legacy_decode_api_client = LegacyDecodeAPIClient::new()?;
        Ok(Self {
            postgres,
            substrate_client,
            legacy_decode_api_client,
        })
    }

    async fn get_metadata(
        &self,
        block_hash_hex: &str,
        spec_version: u32,
    ) -> anyhow::Result<Arc<RuntimeMetadata>> {
        let mut metadata_cache = METADATA_CACHE.write().await;
        let metadata = {
            if !metadata_cache.contains(&spec_version) {
                let metadata =
                    if let Some(db_metadata) = self.postgres.get_metadata(spec_version).await? {
                        db_metadata.1
                    } else {
                        let metadata_hex_string = self
                            .substrate_client
                            .get_metadata_hex_string_at_block(block_hash_hex)
                            .await?;
                        let mut metadata_bytes: &[u8] = &hex::decode(metadata_hex_string)?;
                        let metadata = RuntimeMetadataPrefixed::decode(&mut metadata_bytes)?;
                        let metadata_json = serde_json::to_value(&metadata)?;
                        self.postgres
                            .ingest_metadata(
                                spec_version,
                                get_metadata_version(&metadata.1),
                                &metadata.encode(),
                                &metadata_json,
                            )
                            .await?;
                        metadata.1
                    };
                metadata_cache.put(spec_version, Arc::new(metadata));
            }
            metadata_cache.get(&spec_version).unwrap()
        };
        Ok(metadata.clone())
    }

    pub async fn process_blocks(
        &self,
        start_block_number: u64,
        end_block_number: u64,
    ) -> anyhow::Result<()> {
        log::info!("⚙️ Ingest blocks {start_block_number}-{end_block_number}.");
        for number in start_block_number..=end_block_number {
            log::info!("🔧 Ingesting block {number}. Target {end_block_number}.");
            let hash_hex = self.substrate_client.get_block_hash(number).await?;
            let hash = hex::decode(&hash_hex)?;
            match self.process_block(&hash_hex, number, true).await {
                Ok(_) => {
                    log::info!("🔽 Ingested block {number}.");
                }
                Err(error) => {
                    log::error!("❌ Error while getting traces for block {number}: {error:?}");
                    self.postgres
                        .save_trace_error(&hash, number, &error.to_string())
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn process_block_0(
        &self,
        block_hash: &[u8],
        block_header: &BlockHeader,
        spec_version: u32,
        is_finalized: bool,
    ) -> anyhow::Result<()> {
        let mut tx = self.postgres.connection_pool.begin().await?;
        self.postgres
            .ingest_block(
                block_hash,
                block_header,
                0,
                is_finalized,
                spec_version,
                0,
                0,
                &mut tx,
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn process_events(
        &self,
        block_hash: &[u8],
        block_header: &BlockHeader,
        block_timestamp: u64,
        spec_version: u32,
        metadata_version: u32,
        metadata: &RuntimeMetadata,
        trace: &BlockTrace,
        is_finalized: bool,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<u32> {
        let mut processed_event_count = 0;
        let events_key = get_storage_plain_key("System", "Events");
        // index events
        let mut processed_events_hex = String::new();
        for (trace_index, trace) in trace.events.iter().enumerate() {
            let trace_data = &trace.data_wrapper.data;
            if trace_data.key == events_key && trace_data.value.to_lowercase() != "none" {
                let value = trace_data
                    .value
                    .trim_start_matches("Some(")
                    .trim_end_matches(")");
                let value = match trace_data.method {
                    StorageMethod::Put => {
                        let mut bytes: &[u8] = &hex::decode(value)?;
                        // skip event count
                        let _ = <Compact<u32>>::decode(&mut bytes)?.0;
                        // skip processed events
                        hex::encode(bytes)
                            .trim_start_matches(&processed_events_hex)
                            .to_string()
                    }
                    _ => value.to_string(),
                };
                let mut bytes: &[u8] = &hex::decode(&value)?;
                if metadata_version < 14 {
                    let event = self
                        .legacy_decode_api_client
                        .decode_event(block_hash, spec_version, bytes)
                        .await?;
                    log::info!("Legacy event: {}", serde_json::to_string(&event)?);
                    log::info!(
                        "Legacy event phase: {}",
                        serde_json::to_string(&event.get_phase()?)?
                    );
                    processed_event_count += 1;
                    continue;
                }
                let phase = frame_system::Phase::decode(&mut bytes)?;
                let (phase, extrinsic_index) = match phase {
                    frame_system::Phase::ApplyExtrinsic(extrinsic_index) => {
                        ("ApplyExtrinsic", Some(extrinsic_index))
                    }
                    frame_system::Phase::Finalization => ("Finalization", None),
                    frame_system::Phase::Initialization => ("Initialization", None),
                };
                let pallet_index: u8 = Decode::decode(&mut bytes)?;
                let event_index: u8 = Decode::decode(&mut bytes)?;
                log::info!("Pallet[{pallet_index}] Event[{event_index}]");
                let (pallet_name, event_name) = match &metadata {
                    RuntimeMetadata::V14(metadata) => {
                        let pallet = metadata
                            .pallets
                            .iter()
                            .find(|metadata_pallet| metadata_pallet.index == pallet_index)
                            .expect("Pallet not found in metadata.");

                        let event_type = metadata
                            .types
                            .types
                            .iter()
                            .find(|ty| ty.id == pallet.event.clone().unwrap().ty.id)
                            .expect("Event type not found in pallet.");
                        let event_variant = match &event_type.ty.type_def {
                            scale_info::TypeDef::Variant(variant) => variant
                                .variants
                                .iter()
                                .find(|variant| variant.index == event_index)
                                .unwrap(),
                            _ => {
                                return Err(anyhow::Error::msg(format!(
                                    "Unexpected non-variant event type: {:?}",
                                    event_type.ty.type_def
                                )))
                            }
                        };

                        let mut map = serde_json::Map::new();
                        for event_field in event_variant.fields.iter() {
                            let field_type = metadata
                                .types
                                .types
                                .iter()
                                .find(|metadata_type| metadata_type.id == event_field.ty.id)
                                .expect("Calls type not found in pallet.");
                            let visitor = JsonValueVisitor::new();
                            let value: JsonValue = scale_decode::visitor::decode_with_visitor(
                                &mut bytes,
                                field_type.id,
                                &metadata.types,
                                visitor,
                            )?;
                            if let Some(field_name) = &event_field.name {
                                map.insert(field_name.clone(), value);
                            } else if let Some(type_name) = &event_field.type_name {
                                map.insert(type_name.clone(), value);
                            } else {
                                map.insert("unnamed".to_string(), value);
                            }
                        }
                        let event = JsonValue::Object(map);
                        log::info!("DECODED EVENT :: {}", serde_json::to_string(&event)?);
                        (pallet.name.clone(), event_variant.name.clone())
                    }
                    _ => unimplemented!("Unsupported runtime metadata."),
                };
                log::info!("Event #{processed_event_count} {pallet_name}.{event_name}");

                self.postgres
                    .ingest_event(
                        block_hash,
                        block_header.get_number()?,
                        block_timestamp,
                        spec_version,
                        is_finalized,
                        trace_index as u32,
                        pallet_index,
                        &pallet_name,
                        event_index,
                        &event_name,
                        extrinsic_index,
                        phase,
                        processed_event_count,
                        tx,
                    )
                    .await?;
                if let StorageMethod::Put = trace_data.method {
                    processed_events_hex.push_str(value.as_str());
                }
                processed_event_count += 1;
            }
        }
        Ok(processed_event_count)
    }

    #[allow(clippy::cognitive_complexity)]
    async fn process_block(
        &self,
        block_hash_hex: &str,
        block_number: u64,
        is_finalized: bool,
    ) -> anyhow::Result<()> {
        let block_hash = hex::decode(block_hash_hex)?;
        if self.postgres.block_trace_exists(&block_hash).await? {
            log::info!("🔁 Block {block_number} had already been ingested.");
            return Ok(());
        }
        let block_header = self
            .substrate_client
            .get_block_header(block_hash_hex)
            .await?;
        let spec_version = self
            .substrate_client
            .get_last_runtime_upgrade_info(block_hash_hex)
            .await?
            .spec_version;
        if block_number == 0 {
            self.process_block_0(&block_hash, &block_header, spec_version, is_finalized)
                .await?;
            return Ok(());
        }
        let block_timestamp = self
            .substrate_client
            .get_block_timestamp(block_hash_hex)
            .await?;
        let metadata = self.get_metadata(block_hash_hex, spec_version).await?;
        let metadata_version = get_metadata_version(&metadata);
        let trace = self
            .substrate_client
            .get_block_trace(block_hash_hex)
            .await?;
        let mut tx = self.postgres.connection_pool.begin().await?;
        self.postgres
            .ingest_block_trace(
                &block_hash,
                &block_header,
                is_finalized,
                spec_version,
                &trace,
                &mut tx,
            )
            .await?;

        /* begin :: get extrinsic and event count */
        let extrinsic_count_key = get_storage_plain_key("System", "ExtrinsicCount");
        let event_count_key = get_storage_plain_key("System", "EventCount");
        let mut extrinsic_count: u32 = 0;
        let mut event_count: u32 = 0;
        for trace in trace.events.iter() {
            let trace_data = &trace.data_wrapper.data;
            if trace_data.key == extrinsic_count_key && trace_data.value.to_lowercase() != "none" {
                let value = trace_data
                    .value
                    .trim_start_matches("Some(")
                    .trim_end_matches(")");
                let mut bytes: &[u8] = &hex::decode(value)?;
                extrinsic_count = Decode::decode(&mut bytes)?;
            } else if trace_data.key == event_count_key && trace_data.value.to_lowercase() != "none"
            {
                let value = trace_data
                    .value
                    .trim_start_matches("Some(")
                    .trim_end_matches(")");
                let mut bytes: &[u8] = &hex::decode(value)?;
                event_count = Decode::decode(&mut bytes)?;
            }
        }
        log::info!("{extrinsic_count} exts, {event_count} events");
        /* end :: get extrinsic and event count */

        self.process_events(
            &block_hash,
            &block_header,
            block_timestamp,
            spec_version,
            metadata_version,
            &metadata,
            &trace,
            is_finalized,
            &mut tx,
        )
        .await?;

        /* begin :: process extrinsics */
        let extrinsic_data_root_key = get_storage_plain_key("System", "ExtrinsicData");
        let mut extrinsics = Vec::new();
        let mut trace_extrinsic_index: u32 = 0;
        for (trace_index, trace) in trace.events.iter().enumerate() {
            let trace_data = &trace.data_wrapper.data;
            if !trace_data.key.starts_with(&extrinsic_data_root_key)
                || trace_data.value.to_lowercase() == "none"
            {
                continue;
            }
            let key = trace_data.key.trim_start_matches(&extrinsic_data_root_key);
            let expected_key = hex::encode(storage::hash(
                &StorageHasher::Twox64Concat,
                &trace_extrinsic_index.encode(),
            ));
            if key != expected_key {
                let error_message = format!("Extrinsic {trace_extrinsic_index} data index key does not match the expected value.");
                return Err(anyhow::Error::msg(error_message));
            }
            log::info!("Extrinsic {trace_extrinsic_index} data @ trace {trace_index}");
            let value = trace_data
                .value
                .trim_start_matches("Some(")
                .trim_end_matches(")");
            let mut bytes: &[u8] = &hex::decode(value)?;
            let bytes_vector: Vec<u8> = Decode::decode(&mut bytes)?;
            //let mut bytes: &[u8] = &bytes_vector;
            //let bytes_vector: Vec<u8> = Decode::decode(&mut bytes)?;
            let bytes: &[u8] = &bytes_vector;
            extrinsics.push((Some(trace_index), hex::encode(bytes)));
            trace_extrinsic_index += 1;
        }
        if extrinsics.is_empty() {
            // fall back on RPC
            let block = self.substrate_client.get_block(block_hash_hex).await?;
            block
                .extrinsics
                .iter()
                .for_each(|e| extrinsics.push((None, e.trim_start_matches("0x").to_string())));
        }

        // index extrinsics
        let mut processed_extrinsic_count = 0;
        for extrinsic in extrinsics.iter() {
            // log::info!("EXT {processed_extrinsic_count} HEX :: {}", extrinsic.1);
            let mut bytes: &[u8] = &hex::decode(&extrinsic.1)?;
            let extrinsic_hash = sp_core::blake2_256(bytes);
            log::info!(
                "EXT {processed_extrinsic_count} HASH {}",
                hex::encode(extrinsic_hash)
            );
            if metadata_version < 14 {
                let extrinsic = self
                    .legacy_decode_api_client
                    .decode_extrinsic(&block_hash, spec_version, bytes)
                    .await?;
                log::info!("Legacy extrinsic: {}", serde_json::to_string(&extrinsic)?);
                processed_extrinsic_count += 1;
                continue;
            }
            let bytes_vector: Vec<u8> = Decode::decode(&mut bytes)?;
            let mut bytes: &[u8] = &bytes_vector;
            let signed_version = bytes.read_byte()?;
            let sign_mask = 0b10000000;
            let version_mask = 0b00000100;
            let is_signed = (signed_version & sign_mask) == sign_mask;
            let version = signed_version & version_mask;
            log::info!("TX VERSION {version}");
            let signature = if is_signed {
                let signer = MultiAddress::decode(&mut bytes)?;
                log::info!("SIGNER {signer:?}");
                // let signer = MultiAddress::decode(&mut bytes)?;
                let signature = sp_runtime::MultiSignature::decode(&mut bytes)?;
                let era: sp_runtime::generic::Era = Decode::decode(&mut bytes)?;
                let nonce: Compact<u32> = Decode::decode(&mut bytes)?; // u32
                let tip: Compact<Balance> = Decode::decode(&mut bytes)?;
                let extra: u8 = Decode::decode(&mut bytes)?;
                let signature = Signature {
                    signer,
                    signature,
                    era,
                    nonce: nonce.0,
                    tip: tip.0,
                    extra,
                };
                log::info!("SIGNATURE {signature:?}");
                Some(signature)
            } else {
                None
            };
            let pallet_index = u8::decode(&mut bytes)?;
            let call_index = u8::decode(&mut bytes)?;
            // TODO get module name, call name, parameters JSON
            let (pallet_name, call_name) = match &*metadata {
                RuntimeMetadata::V14(metadata) => {
                    let pallet = metadata
                        .pallets
                        .iter()
                        .find(|metadata_pallet| metadata_pallet.index == pallet_index)
                        .expect("Module not found in metadata.");
                    let calls_type = metadata
                        .types
                        .types
                        .iter()
                        .find(|metadata_type| {
                            metadata_type.id == pallet.calls.clone().unwrap().ty.id
                        })
                        .expect("Calls type not found in pallet.");
                    let call_variant = match &calls_type.ty.type_def {
                        scale_info::TypeDef::Variant(variant) => variant
                            .variants
                            .iter()
                            .find(|variant| variant.index == call_index)
                            .unwrap(),
                        _ => {
                            return Err(anyhow::Error::msg(format!(
                                "Unexpected non-variant call type: {:?}",
                                calls_type.ty.type_def
                            )))
                        }
                    };

                    let mut map = serde_json::Map::new();
                    for call_field in call_variant.fields.iter() {
                        let visitor = JsonValueVisitor::new();
                        let value: JsonValue = scale_decode::visitor::decode_with_visitor(
                            &mut bytes,
                            call_field.ty.id,
                            &metadata.types,
                            visitor,
                        )?;

                        if let Some(field_name) = &call_field.name {
                            map.insert(field_name.clone(), value);
                        } else if let Some(type_name) = &call_field.type_name {
                            map.insert(type_name.clone(), value);
                        } else {
                            map.insert("noname".to_string(), value);
                        }
                    }
                    let extrinsic = JsonValue::Object(map);
                    log::info!(
                        "DECODED EXTRINSIC :: {}",
                        serde_json::to_string(&extrinsic)?
                    );
                    (pallet.name.clone(), call_variant.name.clone())
                }
                _ => unimplemented!("Unsupported runtime metadata."),
            };
            log::info!(
                "Extrinsic #{processed_extrinsic_count} {pallet_name}.{call_name} :: signed? {}",
                signature.is_some(),
            );

            self.postgres
                .ingest_extrinsic(
                    &block_hash,
                    block_header.get_number()?,
                    block_timestamp,
                    spec_version,
                    is_finalized,
                    extrinsic.0.map(|i| i as u32),
                    pallet_index,
                    &pallet_name,
                    call_index,
                    &call_name,
                    &extrinsic_hash,
                    processed_extrinsic_count,
                    version,
                    &signature,
                    true,
                    &mut tx,
                )
                .await?;

            processed_extrinsic_count += 1;
            if processed_extrinsic_count == extrinsic_count {
                break;
            }
        }
        if processed_extrinsic_count < extrinsic_count {
            let error_message = format!("Processed extrinsic count {processed_extrinsic_count} is less than total extrinsic count {extrinsic_count}.");
            return Err(anyhow::Error::msg(error_message));
        }
        /* begin :: process extrinsics */

        self.postgres
            .ingest_block(
                &block_hash,
                &block_header,
                block_timestamp,
                is_finalized,
                spec_version,
                extrinsic_count,
                event_count,
                &mut tx,
            )
            .await?;
        self.postgres
            .ingest_block_logs(&block_hash, &block_header, true, &mut tx)
            .await?;
        self.postgres
            .delete_trace_error(&block_hash, &mut tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
