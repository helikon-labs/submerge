use async_recursion::async_recursion;
use convert_case::{Case, Casing};
use frame_metadata::{v16::StorageHasher, RuntimeMetadata};
use parity_scale_codec::{Decode, Encode, Input};
use rustc_hash::FxHashMap as HashMap;
use serde_json::Value as JSONValue;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::{
    account_id::AccountId, block::BlockHeader, block_trace::BlockTrace,
    multi_address::MultiAddress, signature::Signature,
};
use submerge_util::substrate::storage::{self, get_storage_plain_key};

use super::BlockProcessor;
use crate::{
    persistence::CrystalPostgreSQLStorage,
    types::{
        decode::{Value, ValueVisitor},
        legacy::LegacyCall,
        metadata::util::{
            get_extrinsic_extra_type, get_extrinsic_signature_type,
            get_extrinsic_signer_address_type, get_metadata_version, get_runtime_call_type,
            get_signed_extensions,
        },
        BlockStatus, Call, Event, Extrinsic,
    },
};

const SIGN_MASK: u8 = 0b10000000;
const VERSION_MASK: u8 = 0b00000100;
const SYSTEM_PALLET_NAME: &str = "System";
const EXTRINSIC_SUCCESS_EVENT: &str = "ExtrinsicSuccess";
const SIGNATURE_PREFIX: &str = "01";
const HEX_PREFIX: &str = "0x";
const METADATA_VERSION_LEGACY_THRESHOLD: u32 = 14;
const SOME_PREFIX: &str = "Some(";
const SOME_SUFFIX: &str = ")";

fn is_extrinsic_successful(index: u32, events: &[Event]) -> bool {
    events
        .iter()
        .filter(|e| e.phase == frame_system::Phase::ApplyExtrinsic(index))
        .any(|e| {
            e.pallet_name.eq_ignore_ascii_case(SYSTEM_PALLET_NAME)
                && e.pallet_event_name
                    .eq_ignore_ascii_case(EXTRINSIC_SUCCESS_EVENT)
        })
}

fn decode_extrinsic(
    metadata: &RuntimeMetadata,
    index: u32,
    trace_index: Option<u32>,
    bytes: &mut [u8],
    events: &[Event],
) -> anyhow::Result<Extrinsic> {
    let extrinsic_hash = sp_core::blake2_256(bytes);
    let is_successful = is_extrinsic_successful(index, events);
    let signer_address_type_path = get_extrinsic_signer_address_type(metadata)?
        .ty
        .path
        .segments
        .join("::");
    let signature_type_path = get_extrinsic_signature_type(metadata)?
        .ty
        .path
        .segments
        .join("::");
    let call_type = get_runtime_call_type(metadata)?;
    let mut bytes_slice: &[u8] = bytes;
    let bytes_vector: Vec<u8> = Decode::decode(&mut bytes_slice)?;
    let mut bytes: &[u8] = &bytes_vector;
    let signed_version = bytes.read_byte()?;
    let is_signed = (signed_version & SIGN_MASK) == SIGN_MASK;
    let version = signed_version & VERSION_MASK;
    let signature = if is_signed {
        let signer = match signer_address_type_path.as_str() {
            "sp_core::crypto::AccountId32" => {
                let signer = AccountId::decode(&mut bytes)?;
                MultiAddress::Id(signer)
            },
            "sp_runtime::multiaddress::MultiAddress" => {
                MultiAddress::decode(&mut bytes)?
            }
            "account::AccountId20" => {
                MultiAddress::Address20(Decode::decode(&mut bytes)?)
            }
            _ => anyhow::bail!("Unsupported signer address type in metadata extrinsic signed extensions: {signer_address_type_path}"),
        };
        let signature = match signature_type_path.as_str() {
            "account::EthereumSignature" => {
                sp_runtime::MultiSignature::Ecdsa(<[u8; 65]>::decode(&mut bytes)?.into())
            }
            "sp_runtime::MultiSignature" => sp_runtime::MultiSignature::decode(&mut bytes)?,
            _ => anyhow::bail!("Unsupported signature type: {signature_type_path}"),
        };
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
                                tracing::warn!(
                                    "Extrinsic extras length ({}) doesn't match metadata signed extensions length ({}).",
                                    values.len(),
                                    extensions.len(),
                                );
                            }
                            let mut map = serde_json::Map::<String, JSONValue>::new();
                            for (key, value) in extensions.iter().zip(values) {
                                map.insert(key.to_case(Case::Camel), value.clone().into());
                            }
                            Some(JSONValue::Object(map))
                        }
                        _ => anyhow::bail!("Unexpected non-array type for extrinsic type extras."),
                    }
                }
                RuntimeMetadata::V15(metadata_v15) => {
                    let visitor = ValueVisitor::new(call_type.id, None);
                    let extra_json_array = scale_decode::visitor::decode_with_visitor(
                        &mut bytes,
                        extra_type.id,
                        &metadata_v15.types,
                        visitor,
                    )?;
                    let extensions = get_signed_extensions(metadata)?;
                    extra = match &extra_json_array {
                        Value::Array(values) => {
                            if values.len() != extensions.len() {
                                tracing::warn!(
                                    "Extrinsic extras length ({}) doesn't match metadata signed extensions length ({}).",
                                    values.len(),
                                    extensions.len(),
                                );
                            }
                            let mut map = serde_json::Map::<String, JSONValue>::new();
                            for (key, value) in extensions.iter().zip(values) {
                                map.insert(key.to_case(Case::Camel), value.clone().into());
                            }
                            Some(JSONValue::Object(map))
                        }
                        _ => anyhow::bail!("Unexpected non-array type for extrinsic type extras."),
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
        RuntimeMetadata::V15(metadata_v15) => {
            let value: Value = scale_decode::visitor::decode_with_visitor(
                &mut bytes,
                call_type.id,
                &metadata_v15.types,
                visitor,
            )?;
            match &value {
                Value::Call(call) => (**call).clone(),
                _ => anyhow::bail!("Non-call value for extrinsic call."),
            }
        }
        _ => return Err(anyhow::Error::msg("Unsupported metadata version.")),
    };
    Ok(Extrinsic {
        index,
        trace_index,
        hash: extrinsic_hash,
        signature,
        version,
        is_successful,
        call,
    })
}

impl BlockProcessor {
    #[async_recursion]
    async fn legacy_json_value_to_value(
        &self,
        block_hash: &[u8],
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
                    result.push(
                        self.legacy_json_value_to_value(block_hash, spec_version, value)
                            .await?,
                    );
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
                        let metadata = self.get_parsed_metadata(block_hash, spec_version).await?;
                        let pallet = metadata
                            .get_pallet_by_name(section)
                            .ok_or(anyhow::Error::msg(format!("Pallet {section} not found.")))?;
                        let pallet_call =
                            pallet
                                .get_call_by_name(method)
                                .ok_or(anyhow::Error::msg(format!(
                                "Call {method} not found in pallet {section} in metadata database."
                            )))?;
                        Value::Call(Box::new(Call {
                            pallet_index: pallet.index,
                            pallet_name: pallet.name.clone(),
                            pallet_call_index: pallet_call.index,
                            pallet_call_name: pallet_call.name.clone(),
                            args: self
                                .legacy_json_value_to_value(block_hash, spec_version, args)
                                .await?,
                        }))
                    }
                    _ => {
                        let mut map = HashMap::default();
                        for (key, json_value) in json_map.iter() {
                            map.insert(
                                key.clone(),
                                Box::new(
                                    self.legacy_json_value_to_value(
                                        block_hash,
                                        spec_version,
                                        json_value,
                                    )
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
        block_hash: &[u8],
        spec_version: u32,
        call: &LegacyCall,
    ) -> anyhow::Result<Call> {
        let metadata = self.get_parsed_metadata(block_hash, spec_version).await?;
        let pallet_name = &call.pallet_name;
        let pallet_call_name = &call.pallet_call_name;
        let pallet = metadata
            .get_pallet_by_name(pallet_name)
            .ok_or(anyhow::Error::msg(format!(
                "Pallet {pallet_name} not found in metadata database."
            )))?;
        let pallet_call = pallet
            .get_call_by_name(pallet_call_name)
            .ok_or(anyhow::Error::msg(format!(
                "Call {pallet_call_name} not found in pallet {pallet_name} in metadata database."
            )))?;
        Ok(Call {
            pallet_index: pallet.index,
            pallet_name: pallet.name.clone(),
            pallet_call_index: pallet_call.index,
            pallet_call_name: pallet_call.name.clone(),
            args: self
                .legacy_json_value_to_value(
                    block_hash,
                    spec_version,
                    &JSONValue::Object(call.args.clone()),
                )
                .await?,
        })
    }

    async fn decode_legacy_extrinsic(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        index: u32,
        trace_index: Option<u32>,
        bytes: &[u8],
        events: &[Event],
    ) -> anyhow::Result<Extrinsic> {
        let legacy_decode_api_client = if let Some(client) = &self.legacy_decode_api_client {
            client
        } else {
            anyhow::bail!(
                "Legacy decode API client is not set. legacy_decode_api_url parameter not set."
            );
        };
        let extrinsic_hash = sp_core::blake2_256(bytes);
        let is_successful = is_extrinsic_successful(index, events);
        let extrinsic = legacy_decode_api_client
            .decode_extrinsic(block_hash, spec_version, bytes)
            .await?;
        let signature = match (&extrinsic.signer, &extrinsic.signature) {
            (Some(signer), Some(signature)) => {
                let mut signature_bytes: &[u8] = &hex::decode(format!(
                    "{SIGNATURE_PREFIX}{}",
                    signature.trim_start_matches(HEX_PREFIX)
                ))?;
                let mut extra_map = serde_json::Map::new();
                if let Some(nonce) = &extrinsic.nonce {
                    extra_map.insert("checkNonce".to_string(), JSONValue::String(nonce.clone()));
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

        Ok(Extrinsic {
            index,
            trace_index,
            hash: extrinsic_hash,
            signature,
            version: 0,
            is_successful,
            call: self
                .convert_legacy_call(block_hash, spec_version, &extrinsic.call)
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
        let block = self.substrate_client.get_block(&block_hash_hex).await?;
        let mut raw_extrinsic_bytes = block.get_raw_extrinsic_bytes()?;
        for bytes in raw_extrinsic_bytes.iter_mut() {
            let index = extrinsics.len() as u32;
            if metadata_version < METADATA_VERSION_LEGACY_THRESHOLD {
                let legacy_extrinsic = self
                    .decode_legacy_extrinsic(block_hash, spec_version, index, None, bytes, events)
                    .await?;
                extrinsics.push(legacy_extrinsic);
                continue;
            }
            let extrinsic = decode_extrinsic(metadata, index, None, bytes, events)?;
            extrinsics.push(extrinsic);
        }
        Ok(extrinsics)
    }

    async fn get_raw_extrinsic_bytes_from_trace(
        &self,
        block_hash: &[u8],
        trace: &BlockTrace,
    ) -> anyhow::Result<Vec<(Option<u32>, Vec<u8>)>> {
        let extrinsic_data_root_key =
            hex::encode(get_storage_plain_key(SYSTEM_PALLET_NAME, "ExtrinsicData"));
        let mut raw_extrinsic_bytes = Vec::new();
        let mut trace_extrinsic_index: u32 = 0;
        for (trace_index, trace) in trace.events.iter().enumerate() {
            let trace_data = &trace.data_wrapper.data;
            if !trace_data.key.starts_with(&extrinsic_data_root_key)
                || trace_data.value.eq_ignore_ascii_case("none")
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
            tracing::trace!("Extrinsic {trace_extrinsic_index} data @ trace {trace_index}");
            let value = trace_data
                .value
                .trim_start_matches(SOME_PREFIX)
                .trim_end_matches(SOME_SUFFIX);
            let mut bytes: &[u8] = &hex::decode(value)?;
            let bytes_vector: Vec<u8> = Decode::decode(&mut bytes)?;
            raw_extrinsic_bytes.push((Some(trace_index as u32), bytes_vector));
            trace_extrinsic_index += 1;
        }
        if raw_extrinsic_bytes.is_empty() {
            // fall back on RPC
            let block_hash_hex = hex::encode(block_hash);
            let block = self.substrate_client.get_block(&block_hash_hex).await?;
            for bytes in block.get_raw_extrinsic_bytes()?.iter() {
                raw_extrinsic_bytes.push((None, bytes.clone()))
            }
        }
        Ok(raw_extrinsic_bytes)
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
        for (trace_index, bytes) in self
            .get_raw_extrinsic_bytes_from_trace(block_hash, trace)
            .await?
            .iter_mut()
        {
            let index = extrinsics.len() as u32;
            if metadata_version < METADATA_VERSION_LEGACY_THRESHOLD {
                let legacy_extrinsic = self
                    .decode_legacy_extrinsic(
                        block_hash,
                        spec_version,
                        index,
                        *trace_index,
                        bytes,
                        events,
                    )
                    .await?;
                extrinsics.push(legacy_extrinsic);
                continue;
            }
            let extrinsic = decode_extrinsic(metadata, index, *trace_index, bytes, events)?;
            extrinsics.push(extrinsic);
        }
        Ok(extrinsics)
    }

    pub async fn process_extrinsics(
        &self,
        block_hash: &[u8],
        block_header: &BlockHeader,
        block_timestamp: Option<u64>,
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
                .find_map(|(id, index)| (*index as u32 == extrinsic.index).then_some(*id))
                .ok_or(anyhow::anyhow!(
                    "Database id for extrinsic index {} is not found after batch ingestion.",
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
                extrinsic.is_successful,
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
        block_timestamp: Option<u64>,
        spec_version: u32,
        block_status: BlockStatus,
        extrinsic_id: i64,
        extrinsic: &Extrinsic,
        extrinsic_is_successful: bool,
        parent_call_id: Option<i64>,
        nesting_index: Option<&str>,
        arg: &Value,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        match arg {
            Value::Call(call) => {
                let metadata = self.get_parsed_metadata(block_hash, spec_version).await?;
                let pallet =
                    metadata
                        .get_pallet_by_index(call.pallet_index)
                        .ok_or(anyhow::anyhow!(
                            "Pallet {} with index {} not found in database.",
                            call.pallet_name,
                            call.pallet_index
                        ))?;
                let pallet_call =
                    pallet
                        .get_call_by_index(call.pallet_call_index)
                        .ok_or(anyhow::anyhow!(
                            "Call {} with index {} not found in database.",
                            call.pallet_call_name,
                            call.pallet_call_index
                        ))?;
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
                        pallet_call.id,
                        &call.args.clone().into(),
                        extrinsic_is_successful,
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
                    extrinsic_is_successful,
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
                        extrinsic_is_successful,
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
                        extrinsic_is_successful,
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
