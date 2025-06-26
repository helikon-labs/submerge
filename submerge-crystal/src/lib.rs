#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::persistence::CrystalPostgreSQLStorage;
use async_trait::async_trait;
use frame_metadata::v16::StorageHasher;
use frame_metadata::RuntimeMetadata;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use parity_scale_codec::{Compact, Decode, Encode, Input};
use rustc_hash::FxHashMap as HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use submerge_base::args::{PostgreSQLArgs, RPCArgs};
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_base::types::substrate::{Balance, MultiAddress, Signature};
use submerge_base::BaseService;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::SubstrateClient;
use submerge_util::substrate::storage::{self, get_storage_plain_key};

mod api;
pub mod args;
mod metrics;
mod persistence;

lazy_static! {
    static ref IS_BUSY: AtomicBool = AtomicBool::new(false);
}

async fn get_postgres(args: &PostgreSQLArgs) -> anyhow::Result<PostgreSQLStorage> {
    PostgreSQLStorage::new(
        &args.postgres_host,
        args.postgres_port,
        &args.postgres_username,
        &args.postgres_password,
        &args.postgres_db_name,
        args.postgres_connection_timeout_secs,
        args.postgres_pool_max_connections,
    )
    .await
}

async fn get_substrate(args: &RPCArgs) -> anyhow::Result<SubstrateClient> {
    SubstrateClient::new(
        &args.http_rpc_url,
        &args.ws_rpc_url,
        args.rpc_connection_timeout_secs,
        args.rpc_request_timeout_secs,
    )
    .await
}

pub struct Crystal {
    args: Args,
    _metadata_cache: HashMap<u32, RuntimeMetadata>,
}

impl Crystal {
    pub fn new(args: Args) -> Self {
        Self {
            args,
            _metadata_cache: Default::default(),
        }
    }

    #[allow(clippy::cognitive_complexity)]
    async fn ingest_block(
        postgres: &PostgreSQLStorage,
        substrate_client: &SubstrateClient,
        hash_hex: &str,
        number: u64,
    ) -> anyhow::Result<()> {
        let hash = hex::decode(hash_hex)?;
        if postgres.block_trace_exists(&hash).await? {
            log::info!("🔁 Block {number} had already been ingested.");
            return Ok(());
        }
        let header = substrate_client.get_block_header(hash_hex).await?;
        let timestamp = substrate_client.get_block_timestamp(hash_hex).await?;
        let last_runtime_upgrade = substrate_client
            .get_last_runtime_upgrade_info(hash_hex)
            .await?;
        let metadata = substrate_client.get_metadata_at_block(hash_hex).await?;
        let mut tx = postgres.connection_pool.begin().await?;
        let trace = substrate_client.get_block_trace(hash_hex).await?;
        postgres
            .ingest_block_trace(
                &hash,
                &header,
                true,
                last_runtime_upgrade.spec_version,
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
        let mut processed_extrinsic_count = 0;
        let mut processed_event_count = 0;
        let extrinsic_data_root_key = get_storage_plain_key("System", "ExtrinsicData");
        let events_key = get_storage_plain_key("System", "Events");
        // index events
        for (trace_index, trace) in trace.events.iter().enumerate() {
            let trace_data = &trace.data_wrapper.data;
            if trace_data.key == events_key {
                log::info!("Event {processed_event_count} @ trace {trace_index}.");
                if trace_data.value.to_lowercase() != "none" {
                    let value = trace_data
                        .value
                        .trim_start_matches("Some(")
                        .trim_end_matches(")");
                    let mut bytes: &[u8] = &hex::decode(value)?;
                    let phase = frame_system::Phase::decode(&mut bytes)?;
                    let extrinsic_index = match phase {
                        frame_system::Phase::ApplyExtrinsic(extrinsic_index) => {
                            Some(extrinsic_index)
                        }
                        _ => None,
                    };
                    let pallet_index: u8 = Decode::decode(&mut bytes)?;
                    let event_index: u8 = Decode::decode(&mut bytes)?;
                    // TODO get module name, call name, parameters JSON
                    let (pallet_name, event_name) = match &metadata {
                        RuntimeMetadata::V8(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V9(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V10(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V11(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V12(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V13(_) => ("".to_string(), "".to_string()),
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
                            (pallet.name.clone(), event_variant.name.clone())
                        }
                        RuntimeMetadata::V15(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V16(_) => ("".to_string(), "".to_string()),
                        _ => unimplemented!("Unsupported runtime metadata."),
                    };

                    postgres
                        .ingest_event(
                            &hash,
                            header.get_number()?,
                            timestamp,
                            last_runtime_upgrade.spec_version,
                            true,
                            trace_index as u32,
                            pallet_index,
                            &pallet_name,
                            event_index,
                            &event_name,
                            extrinsic_index,
                            processed_event_count,
                            &mut tx,
                        )
                        .await?;
                }
                processed_event_count += 1;
            }
        }
        // index extrinsics
        for (trace_index, trace) in trace.events.iter().enumerate() {
            let trace_data = &trace.data_wrapper.data;
            if trace_data.key.starts_with(&extrinsic_data_root_key) {
                let key = trace_data.key.trim_start_matches(&extrinsic_data_root_key);
                let expected_key = hex::encode(storage::hash(
                    &StorageHasher::Twox64Concat,
                    &processed_extrinsic_count.encode(),
                ));
                if key != expected_key {
                    let error_message = format!("Extrinsic {processed_extrinsic_count} data index key does not match the expected value.");
                    return Err(anyhow::Error::msg(error_message));
                }
                log::info!("Extrinsic {processed_extrinsic_count} data @ trace {trace_index}");
                if trace_data.value.to_lowercase() != "none" {
                    let value = trace_data
                        .value
                        .trim_start_matches("Some(")
                        .trim_end_matches(")");
                    let mut bytes: &[u8] = &hex::decode(value)?;
                    let bytes_vector: Vec<u8> = Decode::decode(&mut bytes)?;
                    let mut bytes: &[u8] = &bytes_vector;
                    let bytes_vector: Vec<u8> = Decode::decode(&mut bytes)?;
                    let mut bytes: &[u8] = &bytes_vector;
                    let signed_version = bytes.read_byte()?;
                    let sign_mask = 0b10000000;
                    let version_mask = 0b00000100;
                    let is_signed = (signed_version & sign_mask) == sign_mask;
                    let version = signed_version & version_mask;
                    let signature = if is_signed {
                        let signer = MultiAddress::decode(&mut bytes)?;
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
                        Some(signature)
                    } else {
                        None
                    };
                    let pallet_index = u8::decode(&mut bytes)?;
                    let call_index = u8::decode(&mut bytes)?;
                    log::info!(
                        "Extrinsic #{processed_extrinsic_count} ({pallet_index}.{pallet_index}) :: signed :: {}",
                        signature.is_some(),
                    );
                    // TODO get module name, call name, parameters JSON
                    let (pallet_name, call_name) = match &metadata {
                        RuntimeMetadata::V8(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V9(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V10(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V11(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V12(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V13(_) => ("".to_string(), "".to_string()),
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
                            (pallet.name.clone(), call_variant.name.clone())
                        }
                        RuntimeMetadata::V15(_) => ("".to_string(), "".to_string()),
                        RuntimeMetadata::V16(_) => ("".to_string(), "".to_string()),
                        _ => unimplemented!("Unsupported runtime metadata."),
                    };

                    postgres
                        .ingest_extrinsic(
                            &hash,
                            header.get_number()?,
                            timestamp,
                            last_runtime_upgrade.spec_version,
                            true,
                            trace_index as u32,
                            pallet_index,
                            &pallet_name,
                            call_index,
                            &call_name,
                            &[],
                            processed_extrinsic_count,
                            version,
                            &signature,
                            true,
                            &mut tx,
                        )
                        .await?;
                }
                processed_extrinsic_count += 1;
                if processed_extrinsic_count == extrinsic_count {
                    break;
                }
            }
        }
        if processed_extrinsic_count < extrinsic_count {
            let error_message = format!("Processed extrinsic count {processed_extrinsic_count} is less than total extrinsic count {extrinsic_count}.");
            return Err(anyhow::Error::msg(error_message));
        }
        postgres
            .ingest_block(
                &hash,
                &header,
                timestamp,
                true,
                last_runtime_upgrade.spec_version,
                extrinsic_count,
                event_count,
                &mut tx,
            )
            .await?;
        postgres
            .ingest_block_logs(&hash, &header, true, &mut tx)
            .await?;
        postgres.delete_trace_error(&hash, &mut tx).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn ingest_blocks(
        postgres: &PostgreSQLStorage,
        substrate_client: &SubstrateClient,
        start_block_number: u64,
        end_block_number: u64,
    ) -> anyhow::Result<()> {
        log::info!("⚙️ Ingest blocks {start_block_number}-{end_block_number}.");
        for number in start_block_number..=end_block_number {
            log::info!("🔧 Ingesting block {number}. Target {end_block_number}.");
            let hash_hex = substrate_client.get_block_hash(number).await?;
            let hash = hex::decode(&hash_hex)?;
            match Self::ingest_block(postgres, substrate_client, &hash_hex, number).await {
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
                let postgres = get_postgres(&self.args.postgres).await?;
                postgres.ingest_genesis(&chainspec).await?;
                let substrate_client = get_substrate(&self.args.rpc).await?;
                let start_block = self.args.start_block.unwrap_or(1);
                let next_block = if self.args.scan {
                    start_block
                } else {
                    postgres
                        .get_next_block_number(start_block, end_block)
                        .await?
                };
                if next_block < end_block {
                    Self::ingest_blocks(&postgres, &substrate_client, next_block, end_block)
                        .await?;
                } else {
                    log::info!("All blocks in range {start_block}-{end_block} had been ingested.");
                }
                Ok(())
            }
            None => loop {
                let error_cell: Arc<OnceCell<anyhow::Error>> = Arc::new(OnceCell::new());
                let postgres = Arc::new(get_postgres(&self.args.postgres).await?);
                postgres.ingest_genesis(&chainspec).await?;
                let substrate_client = Arc::new(get_substrate(&self.args.rpc).await?);
                substrate_client
                    .subscribe_to_finalized_blocks(
                        self.args.rpc.rpc_request_timeout_secs,
                        |finalized_block_header| {
                            let error_cell = error_cell.clone();
                            let postgres = postgres.clone();
                            let substrate_client = substrate_client.clone();
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
                                    self.args.start_block.unwrap_or(1)
                                } else {
                                    postgres
                                        .get_next_block_number(self.args.start_block.unwrap_or(1), finalized_block_number)
                                        .await?
                                };
                                if start_block <= finalized_block_number {
                                    let postgres = postgres.clone();
                                    let substrate_client = substrate_client.clone();
                                    tokio::spawn(async move {
                                        if let Err(error) = Self::ingest_blocks(
                                            &postgres,
                                            &substrate_client,
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
                tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
            },
        }
    }
}
