#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::legacy::LegacyDecodeAPIClient;
use crate::persistence::CrystalPostgreSQLStorage;
use async_trait::async_trait;
use decode::JsonValueVisitor;
use frame_metadata::v16::StorageHasher;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use lazy_static::lazy_static;
use lru::LruCache;
use once_cell::sync::OnceCell;
use parity_scale_codec::{Compact, Decode, Encode, Input};
use serde_json::Value;
use std::fs;
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use submerge_base::types::substrate::block_trace::StorageMethod;
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_base::types::substrate::{Balance, MultiAddress, Signature};
use submerge_base::BaseService;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::SubstrateClient;
use submerge_util::substrate::storage::{self, get_storage_plain_key};
use tokio::sync::RwLock;
use tokio::time::sleep;

mod api;
pub mod args;
mod decode;
mod legacy;
mod metrics;
mod persistence;

const METADATA_CACHE_SIZE: usize = 50;

lazy_static! {
    static ref IS_BUSY: AtomicBool = AtomicBool::new(false);
}

pub struct Crystal {
    args: Args,
}

impl Crystal {
    pub fn new(args: Args) -> Self {
        Self { args }
    }

    #[allow(clippy::cognitive_complexity)]
    async fn ingest_block(
        postgres: &PostgreSQLStorage,
        substrate_client: &SubstrateClient,
        legacy_decode_api_client: &LegacyDecodeAPIClient,
        metadata_cache: &Arc<RwLock<LruCache<u32, RuntimeMetadata>>>,
        block_hash_hex: &str,
        block_number: u64,
        is_finalized: bool,
    ) -> anyhow::Result<()> {
        let block_hash = hex::decode(block_hash_hex)?;
        if postgres.block_trace_exists(&block_hash).await? {
            log::info!("🔁 Block {block_number} had already been ingested.");
            return Ok(());
        }
        let block_header = substrate_client.get_block_header(block_hash_hex).await?;
        let spec_version = substrate_client
            .get_last_runtime_upgrade_info(block_hash_hex)
            .await?
            .spec_version;
        if block_number == 0 {
            let mut tx = postgres.connection_pool.begin().await?;
            postgres
                .ingest_block(
                    &block_hash,
                    &block_header,
                    0,
                    is_finalized,
                    spec_version,
                    0,
                    0,
                    &mut tx,
                )
                .await?;
            tx.commit().await?;
            return Ok(());
        }
        let mut metadata_cache = metadata_cache.write().await;
        let block_timestamp = substrate_client.get_block_timestamp(block_hash_hex).await?;
        let metadata = {
            if !metadata_cache.contains(&spec_version) {
                let metadata = if let Some(db_metadata_prefixed) =
                    postgres.get_metadata_prefixed(spec_version).await?
                {
                    db_metadata_prefixed.1
                } else {
                    let metadata_hex_string = substrate_client
                        .get_metadata_hex_string_at_block(block_hash_hex)
                        .await?;
                    let mut metadata_bytes: &[u8] = &hex::decode(metadata_hex_string)?;
                    let metadata_prefixed = RuntimeMetadataPrefixed::decode(&mut metadata_bytes)?;
                    let metadata_json = serde_json::to_value(&metadata_prefixed)?;
                    postgres
                        .ingest_metadata_prefixed(
                            spec_version,
                            decode::get_metadata_version(&metadata_prefixed.1),
                            &metadata_prefixed.encode(),
                            &metadata_json,
                        )
                        .await?;
                    metadata_prefixed.1
                };
                metadata_cache.put(spec_version, metadata);
            }
            metadata_cache.get(&spec_version).unwrap()
        };
        let metadata_version = decode::get_metadata_version(metadata);

        let trace = substrate_client.get_block_trace(block_hash_hex).await?;
        let mut tx = postgres.connection_pool.begin().await?;
        postgres
            .ingest_block_trace(
                &block_hash,
                &block_header,
                is_finalized,
                spec_version,
                &trace,
                &mut tx,
            )
            .await?;
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
                    let event = legacy_decode_api_client
                        .decode_event(&block_hash, spec_version, bytes)
                        .await?;
                    log::info!("Legacy event: {}", serde_json::to_string(&event)?);
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
                            let value: Value = scale_decode::visitor::decode_with_visitor(
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
                        let event = Value::Object(map);
                        log::info!("DECODED EVENT :: {}", serde_json::to_string(&event)?);
                        (pallet.name.clone(), event_variant.name.clone())
                    }
                    _ => unimplemented!("Unsupported runtime metadata."),
                };
                log::info!("Event #{processed_event_count} {pallet_name}.{event_name}");

                postgres
                    .ingest_event(
                        &block_hash,
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
                        &mut tx,
                    )
                    .await?;
                if let StorageMethod::Put = trace_data.method {
                    processed_events_hex.push_str(value.as_str());
                }
                processed_event_count += 1;
            }
        }
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
            let block = substrate_client.get_block(block_hash_hex).await?;
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
                let extrinsic = legacy_decode_api_client
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
            let (pallet_name, call_name) = match &metadata {
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
                        let value: Value = scale_decode::visitor::decode_with_visitor(
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
                    let extrinsic = Value::Object(map);
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

            postgres
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
        postgres
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
        postgres
            .ingest_block_logs(&block_hash, &block_header, true, &mut tx)
            .await?;
        postgres.delete_trace_error(&block_hash, &mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn ingest_blocks(
        postgres: &PostgreSQLStorage,
        substrate_client: &SubstrateClient,
        legacy_decode_api_client: &LegacyDecodeAPIClient,
        metadata_cache: &Arc<RwLock<LruCache<u32, RuntimeMetadata>>>,
        start_block_number: u64,
        end_block_number: u64,
    ) -> anyhow::Result<()> {
        log::info!("⚙️ Ingest blocks {start_block_number}-{end_block_number}.");
        for number in start_block_number..=end_block_number {
            log::info!("🔧 Ingesting block {number}. Target {end_block_number}.");
            let hash_hex = substrate_client.get_block_hash(number).await?;
            let hash = hex::decode(&hash_hex)?;
            match Self::ingest_block(
                postgres,
                substrate_client,
                legacy_decode_api_client,
                metadata_cache,
                &hash_hex,
                number,
                true,
            )
            .await
            {
                Ok(_) => {
                    log::info!("🔽 Ingested block {number}.");
                }
                Err(error) => {
                    log::error!("❌ Error while getting traces for block {number}: {error:?}");
                    postgres
                        .save_trace_error(&hash, number, &error.to_string())
                        .await?;
                }
            }
        }
        Ok(())
    }
}

#[async_trait(?Send)]
impl BaseService for Crystal {
    fn get_name(&self) -> String {
        "💠 Submerge Crystal".to_string()
    }

    fn get_metrics_server_addr(&self) -> (String, u16) {
        (
            self.args.metrics.metrics_host.clone(),
            self.args.metrics.metrics_port,
        )
    }

    async fn run(&self) -> anyhow::Result<()> {
        let chainspec_json = fs::read_to_string(&self.args.chainspec_path)?;
        let chainspec: Chainspec = serde_json::from_str(&chainspec_json)?;
        println!(
            r#"┌──────────────────────────────────────────────────────────────────────────────────────────
| Chain:            {}
│ HTTP RPC URL:     {}
│ WS RPC URL:       {}
│ Start Block:      {}
│ End Block:        {}
| API Enabled:      {}
| Metrics Enabled:  {}
└──────────────────────────────────────────────────────────────────────────────────────────"#,
            chainspec.name,
            self.args.rpc.http_rpc_url,
            self.args.rpc.ws_rpc_url,
            self.args
                .start_block
                .map_or("None".to_string(), |v| v.to_string()),
            self.args
                .end_block
                .map_or("None".to_string(), |v| v.to_string()),
            !self.args.no_api,
            !self.args.no_metrics,
        );

        if !self.args.no_api {
            let host = self.args.api.api_host.clone();
            let port = self.args.api.api_port;
            let postgres_args = self.args.postgres.clone();
            tokio::spawn(async move {
                let _ = api::run_api(&postgres_args, host.as_str(), port).await;
            });
        } else {
            log::info!("⛔ API disabled.");
        }

        match self.args.end_block {
            Some(end_block) => {
                let postgres = PostgreSQLStorage::new(&self.args.postgres).await?;
                postgres.ingest_genesis(&chainspec).await?;
                let substrate_client = SubstrateClient::new(&self.args.rpc).await?;
                let legacy_decode_api_client = LegacyDecodeAPIClient::new()?;
                let metadata_cache = Arc::new(RwLock::new(LruCache::<u32, RuntimeMetadata>::new(
                    NonZero::new(METADATA_CACHE_SIZE).unwrap(),
                )));
                let start_block = self.args.start_block.unwrap_or(0);
                let next_block = if self.args.scan {
                    start_block
                } else {
                    postgres
                        .get_next_block_number(start_block, end_block)
                        .await?
                };
                if next_block < end_block {
                    Self::ingest_blocks(
                        &postgres,
                        &substrate_client,
                        &legacy_decode_api_client,
                        &metadata_cache,
                        next_block,
                        end_block,
                    )
                    .await?;
                } else {
                    log::info!("All blocks in range {start_block}-{end_block} had been ingested.");
                }
                Ok(())
            }
            None => loop {
                let error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
                let postgres = Arc::new(PostgreSQLStorage::new(&self.args.postgres).await?);
                postgres.ingest_genesis(&chainspec).await?;
                let substrate_client = Arc::new(SubstrateClient::new(&self.args.rpc).await?);
                let legacy_decode_api_client = Arc::new(LegacyDecodeAPIClient::new()?);
                let metadata_cache = Arc::new(RwLock::new(LruCache::<u32, RuntimeMetadata>::new(
                    NonZero::new(METADATA_CACHE_SIZE).unwrap(),
                )));
                substrate_client
                    .subscribe_to_finalized_blocks(
                        self.args.rpc.rpc_request_timeout_secs,
                        |finalized_block_header| {
                            let error_cell = error_cell.clone();
                            let postgres = postgres.clone();
                            let substrate_client = substrate_client.clone();
                            let legacy_decode_api_client = legacy_decode_api_client.clone();
                            let metadata_cache = metadata_cache.clone();
                            async move {
                                if let Some(error) = error_cell.get() {
                                    return Err(anyhow::anyhow!("{:?}", error));
                                }
                                let finalized_block_number = finalized_block_header.get_number()?;
                                log::info!("📦 New finalized block {finalized_block_number}.");

                                if IS_BUSY.load(Ordering::SeqCst) {
                                    log::info!("⏳ Busy ingesting past blocks. Skip block #{finalized_block_number}.");
                                    return Ok(());
                                }
                                IS_BUSY.store(true, Ordering::SeqCst);

                                let start_block = if self.args.scan {
                                    self.args.start_block.unwrap_or(0)
                                } else {
                                    postgres
                                        .get_next_block_number(self.args.start_block.unwrap_or(0), finalized_block_number)
                                        .await?
                                };
                                if start_block <= finalized_block_number {
                                    tokio::spawn(async move {
                                        if let Err(error) = Self::ingest_blocks(
                                            &postgres,
                                            &substrate_client,
                                            &legacy_decode_api_client,
                                            &metadata_cache,
                                            start_block,
                                            finalized_block_number,
                                        )
                                        .await
                                        {
                                            let _ = error_cell.set(error);
                                        }
                                        IS_BUSY.store(false, Ordering::SeqCst);
                                    });
                                } else {
                                    log::info!("🔁 Block {finalized_block_number} had already been ingested.");
                                    IS_BUSY.store(false, Ordering::SeqCst);
                                }
                                Ok(())
                            }
                        },
                    )
                    .await;
                let delay_seconds = self.args.service.recovery_sleep_seconds;
                log::error!("New block subscription exited. Will refresh connection and subscription after {delay_seconds} seconds.");
                sleep(Duration::from_secs(delay_seconds)).await;
            },
        }
    }
}
