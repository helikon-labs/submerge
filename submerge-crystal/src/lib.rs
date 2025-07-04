#![warn(clippy::disallowed_types)]

use crate::args::Args;
use crate::legacy::LegacyDecodeAPIClient;
use crate::persistence::CrystalPostgreSQLStorage;
use async_trait::async_trait;
use frame_metadata::v14::RuntimeMetadataV14;
use frame_metadata::v16::StorageHasher;
use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use parity_scale_codec::{Compact, Decode, Encode, Input};
use rustc_hash::FxHashMap as HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use submerge_base::args::{PostgreSQLArgs, RPCArgs};
use submerge_base::types::substrate::block_trace::StorageMethod;
use submerge_base::types::substrate::chainspec::Chainspec;
use submerge_base::types::substrate::{Balance, MultiAddress, Signature};
use submerge_base::BaseService;
use submerge_persistence::postgres::PostgreSQLStorage;
use submerge_substrate_client::SubstrateClient;
use submerge_util::substrate::storage::{self, get_storage_plain_key};
use tokio::time::sleep;

mod api;
pub mod args;
mod bits;
mod legacy;
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

pub(crate) fn get_metadata_type(
    metadata: &RuntimeMetadataV14,
    type_id: u32,
) -> &scale_info::Type<scale_info::form::PortableForm> {
    &metadata
        .types
        .types
        .iter()
        .find(|metadata_ty| metadata_ty.id == type_id)
        .unwrap()
        .ty
}

fn decode_bit_sequence(
    bit_store_type: &scale_info::Type<scale_info::form::PortableForm>,
    bit_order_type: &scale_info::Type<scale_info::form::PortableForm>,
    bytes: &mut &[u8],
    json_buffer: &mut Vec<String>,
) -> anyhow::Result<()> {
    let bit_order_type_path = bit_order_type.path.segments.join("::");
    let bit_vector: Vec<u8> = match &bit_store_type.type_def {
        scale_info::TypeDef::Primitive(ty) => match bit_order_type_path.as_str() {
            "bitvec::order::Lsb0" => match ty {
                scale_info::TypeDefPrimitive::U8 => {
                    let bits: bits::DecodedBits<u8, bits::Lsb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                scale_info::TypeDefPrimitive::U16 => {
                    let bits: bits::DecodedBits<u16, bits::Lsb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                scale_info::TypeDefPrimitive::U32 => {
                    let bits: bits::DecodedBits<u32, bits::Lsb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                scale_info::TypeDefPrimitive::U64 => {
                    let bits: bits::DecodedBits<u64, bits::Lsb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                _ => {
                    return Err(anyhow::Error::msg(format!(
                        "Unexpected bit sequence primitive: {ty:?}"
                    )))
                }
            },
            "bitvec::order::Msb0" => match ty {
                scale_info::TypeDefPrimitive::U8 => {
                    let bits: bits::DecodedBits<u8, bits::Msb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                scale_info::TypeDefPrimitive::U16 => {
                    let bits: bits::DecodedBits<u16, bits::Msb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                scale_info::TypeDefPrimitive::U32 => {
                    let bits: bits::DecodedBits<u32, bits::Msb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                scale_info::TypeDefPrimitive::U64 => {
                    let bits: bits::DecodedBits<u64, bits::Msb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                _ => {
                    return Err(anyhow::Error::msg(format!(
                        "Unexpected bit sequence primitive: {ty:?}",
                    )))
                }
            },
            _ => {
                return Err(anyhow::Error::msg(format!(
                    "Unexpected bit sequence order: {bit_order_type_path}"
                )))
            }
        },
        _ => {
            return Err(anyhow::Error::msg(
                "Non-primitive type fed for bit sequence.".to_string(),
            ))
        }
    };
    let hex = hex::encode(&bit_vector);
    //print!("\"{hex}\"");
    json_buffer.push(format!("\"{hex}\""));
    Ok(())
}

fn decode_compact_primitive(
    type_def: &scale_info::TypeDefPrimitive,
    bytes: &mut &[u8],
    json_buffer: &mut Vec<String>,
) -> anyhow::Result<()> {
    match type_def {
        scale_info::TypeDefPrimitive::Bool => {
            return Err(anyhow::Error::msg("No compact for Bool.".to_string()));
        }
        scale_info::TypeDefPrimitive::Char => {
            let value: Compact<u8> = Decode::decode(bytes)?;
            let character = value.0 as char;
            let json_string = serde_json::to_string(&character)?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U8 => {
            let value: Compact<u8> = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.0.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::Str => {
            return Err(anyhow::Error::msg("No compact for Str.".to_string()));
        }
        scale_info::TypeDefPrimitive::U16 => {
            let value: Compact<u16> = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.0.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U32 => {
            let value: Compact<u32> = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.0.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U64 => {
            let value: Compact<u64> = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.0.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U128 => {
            let value: Compact<u128> = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.0.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U256 => {
            return Err(anyhow::Error::msg("No compact for U256."));
        }
        scale_info::TypeDefPrimitive::I8 => {
            return Err(anyhow::Error::msg("No compact for I8.".to_string()));
        }
        scale_info::TypeDefPrimitive::I16 => {
            return Err(anyhow::Error::msg("No compact for I16.".to_string()));
        }
        scale_info::TypeDefPrimitive::I32 => {
            return Err(anyhow::Error::msg("No compact for I32.".to_string()));
        }
        scale_info::TypeDefPrimitive::I64 => {
            return Err(anyhow::Error::msg("No compact for I64.".to_string()));
        }
        scale_info::TypeDefPrimitive::I128 => {
            return Err(anyhow::Error::msg("No compact for I128.".to_string()));
        }
        scale_info::TypeDefPrimitive::I256 => {
            return Err(anyhow::Error::msg("No compact for I256.".to_string()));
        }
    }
    Ok(())
}

fn decode_primitive(
    type_def: &scale_info::TypeDefPrimitive,
    bytes: &mut &[u8],
    json_buffer: &mut Vec<String>,
) -> anyhow::Result<()> {
    match type_def {
        scale_info::TypeDefPrimitive::Bool => {
            let value: bool = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value)?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::Str => {
            let value: String = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value)?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::Char => {
            let value: u8 = Decode::decode(bytes)?;
            let character = value as char;
            let json_string = serde_json::to_string(&character)?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U8 => {
            let value: u8 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U16 => {
            let value: u16 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U32 => {
            let value: u32 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U64 => {
            let value: u64 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U128 => {
            let value: u128 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::U256 => {
            let value: sp_core::U256 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::I8 => {
            let value: i8 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::I16 => {
            let value: i16 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::I32 => {
            let value: i32 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::I64 => {
            let value: i64 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::I128 => {
            let value: i128 = Decode::decode(bytes)?;
            let json_string = serde_json::to_string(&value.to_string())?;
            json_buffer.push(json_string);
        }
        scale_info::TypeDefPrimitive::I256 => {
            let value: [u8; 32] = Decode::decode(bytes)?;
            let hex = hex::encode(value);
            let json_string = serde_json::to_string(&hex)?;
            json_buffer.push(json_string);
        }
    }
    Ok(())
}

fn decode_value(
    metadata: &RuntimeMetadataV14,
    value_type: &scale_info::Type<scale_info::form::PortableForm>,
    bytes: &mut &[u8],
    is_compact: bool,
    sequence_length: Option<u32>,
    json_buffer: &mut Vec<String>,
) -> anyhow::Result<()> {
    match &value_type.type_def {
        scale_info::TypeDef::Primitive(primitive_type_def) => {
            if is_compact {
                decode_compact_primitive(primitive_type_def, bytes, json_buffer)?;
            } else {
                decode_primitive(primitive_type_def, bytes, json_buffer)?;
            }
        }
        scale_info::TypeDef::Composite(composite_type_def) => {
            if composite_type_def.fields.len() == 1
                && composite_type_def.fields.first().unwrap().name.is_none()
            {
            } else {
                json_buffer.push("{".to_string());
            }
            for (i, field) in composite_type_def.fields.iter().enumerate() {
                if let Some(name) = field.name.as_ref() {
                    json_buffer.push(format!("\"{name}\": "));
                }
                let field_type = get_metadata_type(metadata, field.ty.id);
                decode_value(
                    metadata,
                    field_type,
                    bytes,
                    is_compact,
                    sequence_length,
                    json_buffer,
                )?;
                if i < (composite_type_def.fields.len() - 1) {
                    json_buffer.push(", ".to_string());
                }
            }
            if composite_type_def.fields.len() == 1
                && composite_type_def.fields.first().unwrap().name.is_none()
            {
            } else {
                json_buffer.push("}".to_string());
            }
        }
        scale_info::TypeDef::Array(array_type_def) => {
            let element_type = get_metadata_type(metadata, array_type_def.type_param.id);
            if let scale_info::TypeDef::Primitive(scale_info::TypeDefPrimitive::U8) =
                &element_type.type_def
            {
                let length = array_type_def.len as usize;
                if bytes.len() < length {
                    return Err(anyhow::anyhow!(
                        "Not enough bytes to decode [u8; {}]",
                        length
                    ));
                }
                let (bytes_to_decode, remaining) = bytes.split_at(length);
                *bytes = remaining;
                let hex_string = hex::encode(bytes_to_decode);
                json_buffer.push(format!("\"0x{hex_string}\""));
            } else {
                json_buffer.push("[".to_string());
                for i in 0..array_type_def.len {
                    decode_value(
                        metadata,
                        element_type,
                        bytes,
                        is_compact,
                        sequence_length,
                        json_buffer,
                    )?;
                    if i < (array_type_def.len - 1) {
                        json_buffer.push(", ".to_string());
                    }
                }
                json_buffer.push("]".to_string());
            }
        }
        scale_info::TypeDef::Tuple(tuple_type_def) => {
            json_buffer.push("[".to_string());
            for (i, field_type_id) in tuple_type_def.fields.iter().enumerate() {
                let field_type = get_metadata_type(metadata, field_type_id.id);
                decode_value(
                    metadata,
                    field_type,
                    bytes,
                    is_compact,
                    sequence_length,
                    json_buffer,
                )?;
                if i < (tuple_type_def.fields.len() - 1) {
                    json_buffer.push(", ".to_string());
                }
            }
            json_buffer.push("]".to_string());
        }
        scale_info::TypeDef::Compact(compact_type_def) => {
            let compact_type = get_metadata_type(metadata, compact_type_def.type_param.id);
            decode_value(
                metadata,
                compact_type,
                bytes,
                true,
                sequence_length,
                json_buffer,
            )?;
        }
        scale_info::TypeDef::Variant(variant_type_def) => {
            let index: u8 = Decode::decode(bytes)?;
            let variant = &variant_type_def
                .variants
                .iter()
                .find(|v| v.index == index)
                .unwrap();
            if variant.name == "None" {
                json_buffer.push("null".to_string());
            } else if variant.name == "Some" {
                let field = variant.fields.first().unwrap();
                let field_type = get_metadata_type(metadata, field.ty.id);
                decode_value(
                    metadata,
                    field_type,
                    bytes,
                    is_compact,
                    sequence_length,
                    json_buffer,
                )?;
            } else {
                json_buffer.push(format!("{{\"type\": \"{}\", \"value\": [", variant.name));
                for (i, field) in variant.fields.iter().enumerate() {
                    let field_type = get_metadata_type(metadata, field.ty.id);
                    decode_value(
                        metadata,
                        field_type,
                        bytes,
                        is_compact,
                        sequence_length,
                        json_buffer,
                    )?;
                    if i < (variant.fields.len() - 1) {
                        json_buffer.push(", ".to_string());
                    }
                }
                json_buffer.push("]}".to_string());
            }
        }
        scale_info::TypeDef::Sequence(sequence_type_def) => {
            let element_type = get_metadata_type(metadata, sequence_type_def.type_param.id);

            // Check if this is Vec<u8>
            if let scale_info::TypeDef::Primitive(scale_info::TypeDefPrimitive::U8) =
                &element_type.type_def
            {
                let length = if let Some(length) = sequence_length {
                    length as usize
                } else {
                    let compact_length: Compact<u32> = Decode::decode(bytes)?;
                    compact_length.0 as usize
                };
                if bytes.len() < length {
                    return Err(anyhow::anyhow!(
                        "Not enough bytes to decode Vec<u8> of length {}",
                        length
                    ));
                }
                let (bytes_to_decode, remaining) = bytes.split_at(length);
                *bytes = remaining;
                let hex_string = hex::encode(bytes_to_decode);
                json_buffer.push(format!("\"0x{hex_string}\""));
            } else {
                // Not Vec<u8>, decode recursively as normal
                let length = if let Some(length) = sequence_length {
                    length
                } else {
                    let compact_length: Compact<u32> = Decode::decode(bytes)?;
                    compact_length.0
                };
                json_buffer.push("[".to_string());
                for i in 0..length {
                    decode_value(metadata, element_type, bytes, is_compact, None, json_buffer)?;
                    if i < (length - 1) {
                        json_buffer.push(", ".to_string());
                    }
                }
                json_buffer.push("]".to_string());
            }
        }
        scale_info::TypeDef::BitSequence(bit_sequence) => {
            let bit_store_type = &metadata.types.types[bit_sequence.bit_store_type.id as usize].ty;
            let bit_order_type = &metadata.types.types[bit_sequence.bit_order_type.id as usize].ty;
            decode_bit_sequence(bit_store_type, bit_order_type, bytes, json_buffer)?;
        }
    }
    Ok(())
}

fn get_metadata_version(metadata: &RuntimeMetadata) -> u32 {
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
        block_hash_hex: &str,
        block_number: u64,
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
                    true,
                    spec_version,
                    0,
                    0,
                    &mut tx,
                )
                .await?;
            tx.commit().await?;
            return Ok(());
        }
        let legace_decode_api_client = LegacyDecodeAPIClient::new()?;
        let block_timestamp = substrate_client.get_block_timestamp(block_hash_hex).await?;
        let metadata = if let Some(db_metadata) = postgres.get_metadata(spec_version).await? {
            let mut metadata_bytes: &[u8] = &db_metadata;
            let metadata_prefixed = RuntimeMetadataPrefixed::decode(&mut metadata_bytes)?;
            metadata_prefixed.1
        } else {
            let metadata_hex_string = substrate_client
                .get_metadata_hex_string_at_block(block_hash_hex)
                .await?;
            let mut metadata_bytes: &[u8] = &hex::decode(metadata_hex_string)?;
            let metadata_prefixed = RuntimeMetadataPrefixed::decode(&mut metadata_bytes)?;
            postgres
                .ingest_metadata(
                    spec_version,
                    get_metadata_version(&metadata_prefixed.1),
                    &metadata_prefixed.encode(),
                )
                .await?;
            metadata_prefixed.1
        };
        let metadata_version = get_metadata_version(&metadata);

        let trace = substrate_client.get_block_trace(block_hash_hex).await?;
        let mut tx = postgres.connection_pool.begin().await?;
        postgres
            .ingest_block_trace(
                &block_hash,
                &block_header,
                true,
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
                    log::info!("Legacy event.");
                    let response = legace_decode_api_client
                        .decode_event(&block_hash, spec_version, bytes)
                        .await?;
                    log::info!("Legacy event decoded: {response}");
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

                        let mut json_buffer = Vec::new();
                        json_buffer.push("{".to_string());
                        for (index, call_field) in event_variant.fields.iter().enumerate() {
                            let field_type = metadata
                                .types
                                .types
                                .iter()
                                .find(|metadata_type| metadata_type.id == call_field.ty.id)
                                .expect("Calls type not found in pallet.");
                            if let Some(field_name) = &call_field.name {
                                json_buffer.push(format!("\"{field_name}\": "));
                            } else if let Some(type_name) = &call_field.type_name {
                                json_buffer.push(format!("\"{type_name}\": "));
                            } else {
                                json_buffer.push("\"NA\": ".to_string());
                            }
                            decode_value(
                                metadata,
                                &field_type.ty,
                                &mut bytes,
                                false,
                                None,
                                &mut json_buffer,
                            )?;
                            if index < (event_variant.fields.len() - 1) {
                                json_buffer.push(", ".to_string())
                            }
                        }
                        json_buffer.push("}".to_string());
                        let json = json_buffer.join("");
                        log::info!("DECODED EVENT :: {json}");

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
                        true,
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
                log::info!("Legacy extrinsic.");
                let response = legace_decode_api_client
                    .decode_extrinsic(&block_hash, spec_version, bytes)
                    .await?;
                log::info!("Legacy decoded: {response}");
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

                    let mut json_buffer = Vec::new();
                    json_buffer.push("{".to_string());
                    for (index, call_field) in call_variant.fields.iter().enumerate() {
                        let field_type = metadata
                            .types
                            .types
                            .iter()
                            .find(|metadata_type| metadata_type.id == call_field.ty.id)
                            .expect("Calls type not found in pallet.");
                        if let Some(field_name) = &call_field.name {
                            json_buffer.push(format!("\"{field_name}\": "));
                        } else if let Some(type_name) = &call_field.type_name {
                            json_buffer.push(format!("\"{type_name}\": "));
                        } else {
                            json_buffer.push("\"NA\": ".to_string());
                        }
                        decode_value(
                            metadata,
                            &field_type.ty,
                            &mut bytes,
                            false,
                            None,
                            &mut json_buffer,
                        )?;
                        if index < (call_variant.fields.len() - 1) {
                            json_buffer.push(", ".to_string())
                        }
                    }
                    json_buffer.push("}".to_string());
                    let json = json_buffer.join("");
                    log::info!("DECODED :: {json}");

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
                    true,
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
                true,
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
                let start_block = self.args.start_block.unwrap_or(0);
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
                                    self.args.start_block.unwrap_or(0)
                                } else {
                                    postgres
                                        .get_next_block_number(self.args.start_block.unwrap_or(0), finalized_block_number)
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
                sleep(Duration::from_secs(delay_seconds)).await;
            },
        }
    }
}
