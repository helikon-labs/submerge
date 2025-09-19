use async_recursion::async_recursion;
use convert_case::{Case, Casing};
use frame_metadata::{v16::StorageHasher, RuntimeMetadata};
use parity_scale_codec::{Decode, Encode, Input};
use rustc_hash::FxHashMap as HashMap;
use serde_json::Value as JSONValue;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::{
    block::BlockHeader, block_trace::BlockTrace, MultiAddress, Signature,
};
use submerge_util::substrate::storage::{self, get_storage_plain_key};

use super::BlockProcessor;
use crate::{
    persistence::CrystalPostgreSQLStorage,
    types::{
        decode::{Value, ValueVisitor},
        legacy::LegacyCall,
        metadata::util::{
            get_extrinsic_extra_type, get_metadata_version, get_runtime_call_type,
            get_signed_extensions,
        },
        BlockStatus, Call, Event, Extrinsic,
    },
};

impl BlockProcessor {
    #[async_recursion]
    async fn legacy_json_value_to_value(
        &self,
        spec_version: u32,
        json_value: &JSONValue,
    ) -> anyhow::Result<Value> {
        let value = match json_value {
            JSONValue::Null => Value::Null,
            JSONValue::Bool(value) => Value::Bool(*value),
            JSONValue::Number(value) => Value::String(value.to_string()),
            JSONValue::String(value) => Value::String(value.to_string()),
            JSONValue::Array(values) => {
                let mut result = Vec::new();
                for value in values.iter() {
                    result.push(self.legacy_json_value_to_value(spec_version, value).await?);
                }
                Value::Array(result)
            }
            JSONValue::Object(json_map) => {
                match (
                    json_map.get("section"),
                    json_map.get("method"),
                    json_map.get("args"),
                ) {
                    (
                        Some(JSONValue::String(section)),
                        Some(JSONValue::String(method)),
                        Some(args),
                    ) => {
                        let pallet_name = section.to_case(Case::UpperCamel);
                        let pallet_index = if let Some(pallet_index) = self
                            .postgres
                            .get_pallet_index_by_name(spec_version, &pallet_name)
                            .await?
                        {
                            pallet_index
                        } else {
                            anyhow::bail!(format!(
                                "Index for pallet {pallet_name} not found in metadata database."
                            ));
                        };
                        let pallet_call_name = method.to_case(Case::UpperCamel);
                        let pallet_call_index = if let Some(pallet_call_index) = self
                            .postgres
                            .get_pallet_call_index_by_name(
                                spec_version,
                                pallet_index,
                                &pallet_call_name,
                            )
                            .await?
                        {
                            pallet_call_index
                        } else {
                            anyhow::bail!(format!(
                                "Index for call {pallet_name}.{pallet_call_name} not found in metadata database."
                            ))
                        };
                        Value::Call(Box::new(Call {
                            pallet_index,
                            pallet_name,
                            pallet_call_index,
                            pallet_call_name,
                            args: self.legacy_json_value_to_value(spec_version, args).await?,
                        }))
                    }
                    _ => {
                        let mut map = HashMap::default();
                        for (key, json_value) in json_map.iter() {
                            map.insert(
                                key.clone(),
                                Box::new(
                                    self.legacy_json_value_to_value(spec_version, json_value)
                                        .await?,
                                ),
                            );
                        }
                        Value::Object(map)
                    }
                }
            }
        };
        Ok(value)
    }

    pub async fn convert_legacy_call(
        &self,
        spec_version: u32,
        call: &LegacyCall,
    ) -> anyhow::Result<Call> {
        let pallet_name = call.pallet_name.to_case(Case::UpperCamel);
        let pallet_index = if let Some(pallet_index) = self
            .postgres
            .get_pallet_index_by_name(spec_version, &pallet_name)
            .await?
        {
            pallet_index
        } else {
            anyhow::bail!(format!(
                "Index for pallet {pallet_name} not found in metadata database."
            ));
        };
        let pallet_call_name = call.pallet_call_name.to_case(Case::UpperCamel);
        let pallet_call_index = if let Some(pallet_call_index) = self
            .postgres
            .get_pallet_call_index_by_name(spec_version, pallet_index, &pallet_call_name)
            .await?
        {
            pallet_call_index
        } else {
            anyhow::bail!(format!(
                "Index for call {pallet_name}.{pallet_call_name} not found in metadata database."
            ))
        };

        Ok(Call {
            pallet_index,
            pallet_name,
            pallet_call_index,
            pallet_call_name,
            args: self
                .legacy_json_value_to_value(spec_version, &JSONValue::Object(call.args.clone()))
                .await?,
        })
    }

    pub async fn get_extrinsics(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        metadata: &RuntimeMetadata,
        events: &[Event],
    ) -> anyhow::Result<Vec<Extrinsic>> {
        let mut extrinsics = Vec::new();
        let metadata_version = get_metadata_version(metadata);
        let block_hash_hex = hex::encode(block_hash);
        let mut raw_extrinsics = Vec::new();
        let block = self.substrate_client.get_block(&block_hash_hex).await?;
        block
            .extrinsics
            .iter()
            .for_each(|e| raw_extrinsics.push(e.trim_start_matches("0x").to_string()));
        for extrinsic_hex in raw_extrinsics.iter() {
            let mut bytes: &[u8] = &hex::decode(extrinsic_hex)?;
            let extrinsic_hash = sp_core::blake2_256(bytes);
            let is_successful = events
                .iter()
                .filter(|e| e.phase == frame_system::Phase::ApplyExtrinsic(extrinsics.len() as u32))
                .any(|e| {
                    e.pallet_name.to_lowercase() == "system"
                        && e.pallet_event_name.to_lowercase() == "extrinsicsuccess"
                });
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
                let signature = match (&extrinsic.signer, &extrinsic.signature) {
                    (Some(signer), Some(signature)) => {
                        let mut signature_bytes: &[u8] =
                            &hex::decode(format!("01{}", signature.trim_start_matches("0x")))?;
                        let mut extra_map = serde_json::Map::new();
                        if let Some(nonce) = &extrinsic.nonce {
                            extra_map
                                .insert("checkNonce".to_string(), JSONValue::String(nonce.clone()));
                        }
                        if let Some(tip) = &extrinsic.tip {
                            extra_map.insert(
                                "chargeTransactionPayment".to_string(),
                                JSONValue::String(tip.clone()),
                            );
                        }
                        if let Some(era) = &extrinsic.era {
                            extra_map.insert("checkMortality".to_string(), era.clone());
                        }
                        Some(Signature {
                            signer: signer.try_into()?,
                            signature: sp_runtime::MultiSignature::decode(&mut signature_bytes)?,
                            extra: Some(JSONValue::Object(extra_map)),
                        })
                    }
                    _ => None,
                };

                extrinsics.push(Extrinsic {
                    index: extrinsics.len() as u32,
                    trace_index: None,
                    hash: extrinsic_hash,
                    signature,
                    version: 0,
                    is_successful,
                    call: self
                        .convert_legacy_call(spec_version, &extrinsic.call)
                        .await?,
                });
                continue;
            }
            let call_type = get_runtime_call_type(metadata)?;
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
                            let extensions = get_signed_extensions(metadata)?;
                            extra = match &extra_json_array {
                                Value::Array(values) => {
                                    if values.len() != extensions.len() {
                                        anyhow::bail!(format!(
                                            "Signed extensions length ({}) doesn't match extrinsic extras length ({})",
                                            extensions.len(),
                                            values.len(),
                                        ));
                                    }
                                    let mut map = serde_json::Map::<String, JSONValue>::new();
                                    for (key, value) in extensions.iter().zip(values) {
                                        map.insert(key.to_case(Case::Camel), value.clone().into());
                                    }
                                    Some(JSONValue::Object(map))
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

            let call_type = get_runtime_call_type(metadata)?;
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
                trace_index: None,
                hash: extrinsic_hash,
                signature,
                version,
                is_successful,
                call,
            });
        }
        Ok(extrinsics)
    }

    pub async fn get_extrinsics_from_trace(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        metadata: &RuntimeMetadata,
        trace: &BlockTrace,
        events: &[Event],
    ) -> anyhow::Result<Vec<Extrinsic>> {
        let mut extrinsics = Vec::new();
        let metadata_version = get_metadata_version(metadata);
        let call_type = if metadata_version >= 14 {
            Some(get_runtime_call_type(metadata)?)
        } else {
            None
        };
        let block_hash_hex = hex::encode(block_hash);
        let extrinsic_data_root_key = get_storage_plain_key("System", "ExtrinsicData");
        let mut raw_extrinsics = Vec::new();
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
            log::trace!("Extrinsic {trace_extrinsic_index} data @ trace {trace_index}");
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
            let is_successful = events
                .iter()
                .filter(|e| e.phase == frame_system::Phase::ApplyExtrinsic(extrinsics.len() as u32))
                .any(|e| {
                    e.pallet_name.to_lowercase() == "system"
                        && e.pallet_event_name.to_lowercase() == "extrinsicsuccess"
                });
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
                let signature = match (&extrinsic.signer, &extrinsic.signature) {
                    (Some(signer), Some(signature)) => {
                        let mut signature_bytes: &[u8] =
                            &hex::decode(format!("01{}", signature.trim_start_matches("0x")))?;
                        let mut extra_map = serde_json::Map::new();
                        if let Some(nonce) = &extrinsic.nonce {
                            extra_map
                                .insert("checkNonce".to_string(), JSONValue::String(nonce.clone()));
                        }
                        if let Some(tip) = &extrinsic.tip {
                            extra_map.insert(
                                "chargeTransactionPayment".to_string(),
                                JSONValue::String(tip.clone()),
                            );
                        }
                        if let Some(era) = &extrinsic.era {
                            extra_map.insert("checkMortality".to_string(), era.clone());
                        }
                        Some(Signature {
                            signer: signer.try_into()?,
                            signature: sp_runtime::MultiSignature::decode(&mut signature_bytes)?,
                            extra: Some(JSONValue::Object(extra_map)),
                        })
                    }
                    _ => None,
                };

                extrinsics.push(Extrinsic {
                    index: extrinsics.len() as u32,
                    trace_index: maybe_trace_index.map(|i| i as u32),
                    hash: extrinsic_hash,
                    signature,
                    version: 0,
                    is_successful,
                    call: self
                        .convert_legacy_call(spec_version, &extrinsic.call)
                        .await?,
                });
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
                            let visitor = ValueVisitor::new(call_type.unwrap().id, None);
                            let extra_json_array = scale_decode::visitor::decode_with_visitor(
                                &mut bytes,
                                extra_type.id,
                                &metadata_v14.types,
                                visitor,
                            )?;
                            let extensions = get_signed_extensions(metadata)?;
                            extra = match &extra_json_array {
                                Value::Array(values) => {
                                    if values.len() != extensions.len() {
                                        anyhow::bail!(format!(
                                            "Signed extensions length ({}) doesn't match extrinsic extras length ({})",
                                            extensions.len(),
                                            values.len(),
                                        ));
                                    }
                                    let mut map = serde_json::Map::<String, JSONValue>::new();
                                    for (key, value) in extensions.iter().zip(values) {
                                        map.insert(key.to_case(Case::Camel), value.clone().into());
                                    }
                                    Some(JSONValue::Object(map))
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

            let call_type = get_runtime_call_type(metadata)?;
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
        block_status: BlockStatus,
        extrinsics: &[Extrinsic],
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let block_number = block_header.get_number()?;
        let ids_to_indices = self
            .postgres
            .ingest_extrinsics(
                block_hash,
                block_number,
                block_timestamp,
                spec_version,
                block_status,
                extrinsics,
                tx,
            )
            .await?;
        for extrinsic in extrinsics.iter() {
            let extrinsic_id = ids_to_indices
                .iter()
                .find_map(|id_to_index| {
                    if id_to_index.1 as u32 == extrinsic.index {
                        Some(id_to_index.0)
                    } else {
                        None
                    }
                })
                .ok_or(anyhow::anyhow!(
                    "Database for extrinsic index {} is not found after batch ingestion.",
                    extrinsic.index
                ))?;
            self.process_extrinsic_arg(
                block_hash,
                block_number,
                block_timestamp,
                spec_version,
                block_status,
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
        block_status: BlockStatus,
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
                        block_status,
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
                    block_status,
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
                        block_status,
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
                        block_status,
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
