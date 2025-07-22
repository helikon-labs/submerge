use async_recursion::async_recursion;
use convert_case::{Case, Casing};
use frame_metadata::{v16::StorageHasher, RuntimeMetadata};
use parity_scale_codec::{Decode, Encode, Input};
use serde_json::Value as JsonValue;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::{
    block::BlockHeader, block_trace::BlockTrace, MultiAddress, Signature,
};
use submerge_util::substrate::storage::{self, get_storage_plain_key};

use crate::{
    persistence::CrystalPostgreSQLStorage,
    process::{
        decode::{Value, ValueVisitor},
        BlockProcessor,
    },
    types::{
        metadata::util::{
            get_call_type, get_extrinsic_extra_type, get_metadata_version, get_signed_extensions,
        },
        Event, Extrinsic,
    },
};

impl BlockProcessor {
    pub async fn get_extrinsics(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        metadata: &RuntimeMetadata,
        trace: &BlockTrace,
        events: &[Event],
    ) -> anyhow::Result<Vec<Extrinsic>> {
        let mut extrinsics = Vec::new();
        let metadata_version = get_metadata_version(metadata);
        let block_hash_hex = hex::encode(block_hash);
        let extrinsic_data_root_key = get_storage_plain_key("System", "ExtrinsicData");
        let mut raw_extrinsics = Vec::new();
        let mut trace_extrinsic_index: u32 = 0;
        let call_type = get_call_type(metadata)?;
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
            let bytes: &[u8] = &bytes_vector;
            raw_extrinsics.push((Some(trace_index), hex::encode(bytes)));
            trace_extrinsic_index += 1;
        }
        if raw_extrinsics.is_empty() {
            // fall back on RPC
            let block = self.substrate_client.get_block(&block_hash_hex).await?;
            block
                .extrinsics
                .iter()
                .for_each(|e| raw_extrinsics.push((None, e.trim_start_matches("0x").to_string())));
        }
        for (maybe_trace_index, extrinsic_hex) in raw_extrinsics.iter() {
            let mut bytes: &[u8] = &hex::decode(extrinsic_hex)?;
            let extrinsic_hash = sp_core::blake2_256(bytes);
            if metadata_version < 14 {
                let legacy_decode_api_client = if let Some(client) = &self.legacy_decode_api_client
                {
                    client
                } else {
                    anyhow::bail!("Legacy decode API client is not set. legacy_decode_api_url parameter not set.");
                };
                let extrinsic = legacy_decode_api_client
                    .decode_extrinsic(block_hash, spec_version, bytes)
                    .await?;

                // get signature, extra
                log::info!("Legacy extrinsic: {}", serde_json::to_string(&extrinsic)?);
                continue;
            }
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
                let mut extra = None;
                if let Some(extra_type) = get_extrinsic_extra_type(metadata)? {
                    match metadata {
                        RuntimeMetadata::V14(metadata_v14) => {
                            let visitor = ValueVisitor::new(call_type.id, None);
                            let extra_json_array = scale_decode::visitor::decode_with_visitor(
                                &mut bytes,
                                extra_type.id,
                                &metadata_v14.types,
                                visitor,
                            )?;
                            let extensions = get_signed_extensions(metadata_v14);
                            extra = match &extra_json_array {
                                Value::Array(values) => {
                                    if values.len() != extensions.len() {
                                        anyhow::bail!(format!(
                                            "Signed extensions length ({}) doesn't match extrinsic extras length ({})",
                                            extensions.len(),
                                            values.len(),
                                        ));
                                    }
                                    let mut map = serde_json::Map::<String, JsonValue>::new();
                                    for (key, value) in extensions.iter().zip(values) {
                                        map.insert(key.to_case(Case::Camel), value.clone().into());
                                    }
                                    Some(JsonValue::Object(map))
                                }
                                _ => anyhow::bail!(
                                    "Unexpected non-array type for extrinsic type extras."
                                ),
                            }
                        }
                        _ => anyhow::bail!(format!(
                            "Unsupported metadata version: {}",
                            get_metadata_version(metadata)
                        )),
                    }
                }
                let signature = Signature {
                    signer,
                    signature,
                    extra,
                };
                Some(signature)
            } else {
                None
            };
            let is_successful = events
                .iter()
                .filter(|e| e.phase == frame_system::Phase::ApplyExtrinsic(extrinsics.len() as u32))
                .any(|e| {
                    e.pallet_name.to_lowercase() == "system"
                        && e.pallet_event_name.to_lowercase() == "extrinsicsuccess"
                });

            let call_type = get_call_type(metadata)?;
            let visitor = ValueVisitor::new(call_type.id, None);
            let call = match metadata {
                RuntimeMetadata::V14(metadata_v14) => {
                    let value: Value = scale_decode::visitor::decode_with_visitor(
                        &mut bytes,
                        call_type.id,
                        &metadata_v14.types,
                        visitor,
                    )?;
                    match &value {
                        Value::Call(call) => (**call).clone(),
                        _ => anyhow::bail!("Non-call value for extrinsic call."),
                    }
                }
                _ => return Err(anyhow::Error::msg("Unsupported metadata version.")),
            };
            extrinsics.push(Extrinsic {
                index: extrinsics.len() as u32,
                trace_index: maybe_trace_index.map(|i| i as u32),
                hash: extrinsic_hash,
                signature,
                version,
                is_successful,
                call,
            });
        }
        Ok(extrinsics)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn process_extrinsics(
        &self,
        block_hash: &[u8],
        block_header: &BlockHeader,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
        extrinsics: &[Extrinsic],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        for extrinsic in extrinsics.iter() {
            let block_number = block_header.get_number()?;
            let extrinsic_id = self
                .postgres
                .ingest_extrinsic(
                    block_hash,
                    block_number,
                    block_timestamp,
                    spec_version,
                    is_finalized,
                    extrinsic,
                    tx,
                )
                .await?;
            self.process_extrinsic_arg(
                block_hash,
                block_number,
                block_timestamp,
                spec_version,
                is_finalized,
                extrinsic_id,
                extrinsic,
                None,
                None,
                &Value::Call(Box::new(extrinsic.call.clone())),
                tx,
            )
            .await?;
        }
        Ok(())
    }

    #[async_recursion]
    #[allow(clippy::too_many_arguments)]
    pub async fn process_extrinsic_arg(
        &self,
        block_hash: &[u8],
        block_number: u64,
        block_timestamp: u64,
        spec_version: u32,
        is_finalized: bool,
        extrinsic_id: i64,
        extrinsic: &Extrinsic,
        parent_call_id: Option<i64>,
        nesting_index: Option<&str>,
        arg: &Value,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        match arg {
            Value::Call(call) => {
                let call_id = self
                    .postgres
                    .ingest_call(
                        block_hash,
                        block_number,
                        block_timestamp,
                        spec_version,
                        is_finalized,
                        extrinsic_id,
                        extrinsic.index,
                        &extrinsic.hash,
                        parent_call_id,
                        nesting_index,
                        call.pallet_index,
                        &call.pallet_name,
                        call.pallet_call_index,
                        &call.pallet_call_name,
                        true,
                        &call.args.clone().into(),
                        tx,
                    )
                    .await?;
                self.process_extrinsic_arg(
                    block_hash,
                    block_number,
                    block_timestamp,
                    spec_version,
                    is_finalized,
                    extrinsic_id,
                    extrinsic,
                    Some(call_id),
                    nesting_index,
                    &call.args,
                    tx,
                )
                .await?;
            }
            Value::Array(values) => {
                for (i, value) in values.iter().enumerate() {
                    let nesting_index = if let Some(nesting_index) = nesting_index {
                        format!("{nesting_index}::{i}")
                    } else {
                        i.to_string()
                    };
                    self.process_extrinsic_arg(
                        block_hash,
                        block_number,
                        block_timestamp,
                        spec_version,
                        is_finalized,
                        extrinsic_id,
                        extrinsic,
                        parent_call_id,
                        Some(&nesting_index),
                        value,
                        tx,
                    )
                    .await?;
                }
            }
            Value::Object(hash_map) => {
                for (key, value) in hash_map.iter() {
                    let nesting_index = if let Some(nesting_index) = nesting_index {
                        format!("{nesting_index}::{key}")
                    } else {
                        key.to_owned()
                    };
                    self.process_extrinsic_arg(
                        block_hash,
                        block_number,
                        block_timestamp,
                        spec_version,
                        is_finalized,
                        extrinsic_id,
                        extrinsic,
                        parent_call_id,
                        Some(&nesting_index),
                        value,
                        tx,
                    )
                    .await?;
                }
            }
            _ => (),
        }

        Ok(())
    }
}
