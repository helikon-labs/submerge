#![warn(clippy::disallowed_types)]
use async_trait::async_trait;
use bits::{DecodedBits, Lsb0, Msb0};
use frame_metadata::v15::StorageEntryType;
use frame_metadata::{
    v14::RuntimeMetadataV14, v15::StorageHasher, RuntimeMetadata, RuntimeMetadataPrefixed,
};
use hex::FromHexError;
use lazy_static::lazy_static;
use parity_scale_codec::{Compact, Decode, Encode};
use rustc_hash::FxHashMap as HashMap;
use scale_info::{form::PortableForm, Type, TypeDefPrimitive};
use sp_core::U256;
use submerge_base::BaseService;
use submerge_config::Config;
use submerge_persistence::postgres::new_postgres_connection_pool;
use submerge_substrate_client::SubstrateClient;
use submerge_types::substrate::block_trace::StorageMethod;

mod bits;
mod persistence;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}

#[derive(Default)]
pub struct Fractal;

#[async_trait(?Send)]
impl BaseService for Fractal {
    fn get_metrics_server_addr() -> (&'static str, u16) {
        (CONFIG.metrics.host.as_str(), CONFIG.metrics.crystal_port)
    }

    async fn run(&'static self) -> anyhow::Result<()> {
        log::info!(":: Submerge Fractal ::");
        let postgres = new_postgres_connection_pool(&CONFIG).await?;
        let _block_number = 541;
        let substrate_client =
            SubstrateClient::new("wss://rpc.helikon.io/coretime-westend-dev", 30, 30).await?;
        for block_number in 1..1000 {
            let block_hash = substrate_client.get_block_hash(block_number).await?;
            let metadata_prefixed = substrate_client.get_metadata_at_block(&block_hash).await?;
            let block_traces = persistence::get_block_traces(&postgres, &block_hash).await?;
            log::info!(
                "There are {} traces in block {block_number} - {block_hash}.",
                block_traces.len()
            );

            let mut traces: HashMap<String, Vec<Option<String>>> = HashMap::default();
            for block_trace in block_traces.iter() {
                let value = if block_trace.value.to_lowercase() == "none" {
                    None
                } else {
                    Some(
                        block_trace
                            .value
                            .trim_start_matches("Some(")
                            .trim_end_matches(")")
                            .to_string(),
                    )
                };
                let key = block_trace.key.as_str();
                if key == "26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7" {
                    log::info!("System.Events :: {} :: {:?}", block_trace.method, value)
                }
                match block_trace.method {
                    StorageMethod::Put => {
                        traces.insert(key.to_string(), vec![value]);
                    }
                    StorageMethod::Append => {
                        assert!(traces.contains_key(key), "Append before Put.");
                        traces.get_mut(key).unwrap().push(value);
                    }
                    _ => log::warn!("Skip trace for now: {}", block_trace.method),
                }
            }

            for key in traces.keys() {
                let values = traces.get(key).unwrap();
                process_trace(&metadata_prefixed, &key, values)?;
            }
        }
        Ok(())
    }
}

pub fn hash(hasher: &StorageHasher, bytes: &[u8]) -> Vec<u8> {
    match hasher {
        StorageHasher::Identity => bytes.to_vec(),
        StorageHasher::Blake2_128 => sp_core::blake2_128(bytes).to_vec(),
        StorageHasher::Blake2_128Concat => sp_core::blake2_128(bytes)
            .iter()
            .chain(bytes)
            .cloned()
            .collect(),
        StorageHasher::Blake2_256 => sp_core::blake2_256(bytes).to_vec(),
        StorageHasher::Twox128 => sp_core::twox_128(bytes).to_vec(),
        StorageHasher::Twox256 => sp_core::twox_256(bytes).to_vec(),
        StorageHasher::Twox64Concat => sp_core::twox_64(bytes)
            .iter()
            .chain(bytes)
            .cloned()
            .collect(),
    }
}

pub fn get_storage_plain_key(module_name: &str, storage_name: &str) -> String {
    let hasher = StorageHasher::Twox128;
    let mut storage_hash: Vec<u8> = Vec::new();
    let mut module_name_hash = hash(&hasher, module_name.as_bytes());
    storage_hash.append(&mut module_name_hash);
    let mut storage_name_hash = hash(&hasher, storage_name.as_bytes());
    storage_hash.append(&mut storage_name_hash);
    hex::encode(storage_hash)
}

pub(crate) fn get_metadata_type(
    metadata: &RuntimeMetadataV14,
    type_id: u32,
) -> &Type<PortableForm> {
    &metadata
        .types
        .types
        .iter()
        .find(|metadata_ty| metadata_ty.id == type_id)
        .unwrap()
        .ty
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum DecodeError {
    #[error("Decode error: {0}")]
    Error(String),
}

impl From<FromHexError> for DecodeError {
    fn from(error: FromHexError) -> Self {
        Self::Error(error.to_string())
    }
}

impl From<parity_scale_codec::Error> for DecodeError {
    fn from(error: parity_scale_codec::Error) -> Self {
        Self::Error(error.to_string())
    }
}

fn decode_bit_sequence(
    bit_store_type: &Type<PortableForm>,
    bit_order_type: &Type<PortableForm>,
    bytes: &mut &[u8],
    json_buffer: &mut Vec<String>,
) -> anyhow::Result<()> {
    let bit_order_type_path = bit_order_type.path.segments.join("::");
    let bit_vector: Vec<u8> = match &bit_store_type.type_def {
        scale_info::TypeDef::Primitive(ty) => match bit_order_type_path.as_str() {
            "bitvec::order::Lsb0" => match ty {
                TypeDefPrimitive::U8 => {
                    let bits: DecodedBits<u8, Lsb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                TypeDefPrimitive::U16 => {
                    let bits: DecodedBits<u16, Lsb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                TypeDefPrimitive::U32 => {
                    let bits: DecodedBits<u32, Lsb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                TypeDefPrimitive::U64 => {
                    let bits: DecodedBits<u64, Lsb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                _ => {
                    return Err(DecodeError::Error(format!(
                        "Unexpected bit sequence primitive: {:?}",
                        ty,
                    ))
                    .into())
                }
            },
            "bitvec::order::Msb0" => match ty {
                TypeDefPrimitive::U8 => {
                    let bits: DecodedBits<u8, Msb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                TypeDefPrimitive::U16 => {
                    let bits: DecodedBits<u16, Msb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                TypeDefPrimitive::U32 => {
                    let bits: DecodedBits<u32, Msb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                TypeDefPrimitive::U64 => {
                    let bits: DecodedBits<u64, Msb0> = Decode::decode(bytes)?;
                    bits.as_bits().encode()
                }
                _ => {
                    return Err(DecodeError::Error(format!(
                        "Unexpected bit sequence primitive: {:?}",
                        ty,
                    ))
                    .into())
                }
            },
            _ => {
                return Err(DecodeError::Error(format!(
                    "Unexpected bit sequence order: {}",
                    bit_order_type_path,
                ))
                .into())
            }
        },
        _ => {
            return Err(
                DecodeError::Error("Non-primitive type fed for bit sequence.".to_string()).into(),
            )
        }
    };
    let hex = hex::encode(&bit_vector);
    print!("\"{hex}\"");
    json_buffer.push(format!("\"{hex}\""));
    Ok(())
}

fn decode_compact_primitive(type_def: &TypeDefPrimitive, bytes: &mut &[u8], json_buffer: &mut Vec<String>) -> anyhow::Result<()> {
    match type_def {
        TypeDefPrimitive::Bool => {
            return Err(DecodeError::Error("No compact for Bool.".to_string()).into());
        }
        TypeDefPrimitive::Char => {
            let value: Compact<u8> = Decode::decode(bytes)?;
            let character = value.0 as char;
            print!("{}", character);
            json_buffer.push(character.to_string());
        }
        TypeDefPrimitive::U8 => {
            let value: Compact<u8> = Decode::decode(bytes)?;
            print!("{}", value.0);
            json_buffer.push(value.0.to_string());
        }
        TypeDefPrimitive::Str => {
            return Err(DecodeError::Error("No compact for Str.".to_string()).into());
        }
        TypeDefPrimitive::U16 => {
            let value: Compact<u16> = Decode::decode(bytes)?;
            print!("{}", value.0);
            json_buffer.push(value.0.to_string());
        }
        TypeDefPrimitive::U32 => {
            let value: Compact<u32> = Decode::decode(bytes)?;
            print!("{}", value.0);
            json_buffer.push(value.0.to_string());
        }
        TypeDefPrimitive::U64 => {
            let value: Compact<u64> = Decode::decode(bytes)?;
            print!("{}", value.0);
            json_buffer.push(value.0.to_string());
        }
        TypeDefPrimitive::U128 => {
            let value: Compact<u128> = Decode::decode(bytes)?;
            print!("{}", value.0);
            json_buffer.push(value.0.to_string());
        }
        TypeDefPrimitive::U256 => {
            return Err(DecodeError::Error("No compact for U256.".to_string()).into());
        }
        TypeDefPrimitive::I8 => {
            return Err(DecodeError::Error("No compact for I8.".to_string()).into());
        }
        TypeDefPrimitive::I16 => {
            return Err(DecodeError::Error("No compact for I16.".to_string()).into());
        }
        TypeDefPrimitive::I32 => {
            return Err(DecodeError::Error("No compact for I32.".to_string()).into());
        }
        TypeDefPrimitive::I64 => {
            return Err(DecodeError::Error("No compact for I64.".to_string()).into());
        }
        TypeDefPrimitive::I128 => {
            return Err(DecodeError::Error("No compact for I128.".to_string()).into());
        }
        TypeDefPrimitive::I256 => {
            return Err(DecodeError::Error("No compact for I256.".to_string()).into());
        }
    }
    Ok(())
}

fn decode_primitive(type_def: &TypeDefPrimitive, bytes: &mut &[u8], json_buffer: &mut Vec<String>) -> anyhow::Result<()> {
    match type_def {
        TypeDefPrimitive::Bool => {
            let value: bool = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::Str => {
            let value: String = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value);
        }
        TypeDefPrimitive::Char => {
            let value: u8 = Decode::decode(bytes)?;
            let character = value as char;
            print!("{}", character);
            json_buffer.push(character.to_string());
        }
        TypeDefPrimitive::U8 => {
            let value: u8 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::U16 => {
            let value: u16 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::U32 => {
            let value: u32 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::U64 => {
            let value: u64 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::U128 => {
            let value: u128 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::U256 => {
            let value: U256 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::I8 => {
            let value: i8 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::I16 => {
            let value: i16 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::I32 => {
            let value: i32 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::I64 => {
            let value: i64 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::I128 => {
            let value: i128 = Decode::decode(bytes)?;
            print!("{value}");
            json_buffer.push(value.to_string());
        }
        TypeDefPrimitive::I256 => {
            let value: [u8; 32] = Decode::decode(bytes)?;
            let hex = hex::encode(value);
            print!("\"{hex}\"");
            json_buffer.push(format!("\"{hex}\""));
        }
    }
    Ok(())
}

fn decode_value(
    metadata: &RuntimeMetadataV14,
    value_type: &Type<PortableForm>,
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
            //log::info!("  composite {:?}", composite_type_def);
            if composite_type_def.fields.len() == 1
                && composite_type_def.fields.first().unwrap().name.is_none()
            {
            } else {
                print!("{{");
                json_buffer.push("{".to_string());
            }
            for (i, field) in composite_type_def.fields.iter().enumerate() {
                //log::info!("  composite field {:?}", field.name);
                if let Some(name) = field.name.as_ref() {
                    print!("\"{}\": ", name);
                    json_buffer.push(format!("\"{}\": ", name));
                }
                let field_type = get_metadata_type(metadata, field.ty.id);
                decode_value(metadata, field_type, bytes, is_compact, sequence_length, json_buffer)?;
                if i < (composite_type_def.fields.len() - 1) {
                    print!(", ");
                    json_buffer.push(", ".to_string());
                }
            }
            if composite_type_def.fields.len() == 1
                && composite_type_def.fields.first().unwrap().name.is_none()
            {
            } else {
                print!("}}");
                json_buffer.push("}".to_string());
            }
        }
        scale_info::TypeDef::Array(array_type_def) => {
            let array_type = get_metadata_type(metadata, array_type_def.type_param.id);
            print!("[");
            json_buffer.push("[".to_string());
            for i in 0..array_type_def.len {
                decode_value(metadata, array_type, bytes, is_compact, sequence_length, json_buffer)?;
                if i < (array_type_def.len - 1) {
                    print!(", ");
                    json_buffer.push(", ".to_string());
                }
            }
            print!("]");
            json_buffer.push("]".to_string());
        }
        scale_info::TypeDef::Tuple(tuple_type_def) => {
            print!("[");
            json_buffer.push("[".to_string());
            for (i, field_type_id) in tuple_type_def.fields.iter().enumerate() {
                let field_type = get_metadata_type(metadata, field_type_id.id);
                decode_value(metadata, field_type, bytes, is_compact, sequence_length, json_buffer)?;
                if i < (tuple_type_def.fields.len() - 1) {
                    print!(", ");
                    json_buffer.push(", ".to_string());
                }
            }
            print!("]");
            json_buffer.push("]".to_string());
        }
        scale_info::TypeDef::Compact(compact_type_def) => {
            let compact_type = get_metadata_type(metadata, compact_type_def.type_param.id);
            decode_value(metadata, compact_type, bytes, true, sequence_length, json_buffer)?;
        }
        scale_info::TypeDef::Variant(variant_type_def) => {
            let index: u8 = Decode::decode(bytes)?;
            let variant = &variant_type_def
                .variants
                .iter()
                .find(|v| v.index == index)
                .unwrap();
            if variant.name == "None" {
                print!("null");
                json_buffer.push("null".to_string());
            } else if variant.name == "Some" {
                let field = variant.fields.get(0).unwrap();
                let field_type = get_metadata_type(metadata, field.ty.id);
                decode_value(metadata, field_type, bytes, is_compact, sequence_length, json_buffer)?;
            } else {
                print!("{{\"type\": \"{}\", \"value\": [", variant.name);
                json_buffer.push(format!("{{\"type\": \"{}\", \"value\": [", variant.name));
                for (i, field) in variant.fields.iter().enumerate() {
                    let field_type = get_metadata_type(metadata, field.ty.id);
                    decode_value(metadata, field_type, bytes, is_compact, sequence_length, json_buffer)?;
                    if i < (variant.fields.len() - 1) {
                        print!(", ");
                        json_buffer.push(", ".to_string());
                    }
                }
                print!("]}}");
                json_buffer.push("]}".to_string());
            }
        }
        scale_info::TypeDef::Sequence(sequence_type_def) => {
            let sequence_type = get_metadata_type(metadata, sequence_type_def.type_param.id);
            let length = if let Some(length) = sequence_length {
                length
            } else {
                let compact_length: Compact<u32> = Decode::decode(bytes)?;
                compact_length.0
            };
            print!("[");
            json_buffer.push("[".to_string());
            for i in 0..length {
                decode_value(metadata, sequence_type, bytes, is_compact, None, json_buffer)?;
                if i < (length - 1) {
                    print!(", ");
                    json_buffer.push(", ".to_string());
                }
            }
            print!("]");
            json_buffer.push("]".to_string());
        }
        scale_info::TypeDef::BitSequence(bit_sequence) => {
            //log::info!("  bit sequence");
            let bit_store_type = &metadata.types.types[bit_sequence.bit_store_type.id as usize].ty;
            let bit_order_type = &metadata.types.types[bit_sequence.bit_order_type.id as usize].ty;
            decode_bit_sequence(bit_store_type, bit_order_type, bytes, json_buffer)?;
        }
    }
    Ok(())
}

fn process_trace(
    metadata_prefixed: &RuntimeMetadataPrefixed,
    trace_key: &str,
    trace_values: &[Option<String>],
) -> anyhow::Result<()> {
    let metadata = match &metadata_prefixed.1 {
        RuntimeMetadata::V14(metadata) => metadata,
        _ => {
            log::warn!("Unsupported metadata version.");
            return Ok(());
        }
    };
    let mut json_buffer = Vec::new();
    // let byte_vector: Vec<u8> = Decode::decode(&mut raw_bytes).unwrap();
    // let mut bytes: &[u8] = byte_vector.as_ref();
    let mut is_found = false;
    'outer: for pallet in metadata.pallets.iter() {
        if let Some(storage) = pallet.storage.as_ref() {
            for storage_entry in storage.entries.iter() {
                let type_id = match &storage_entry.ty {
                    StorageEntryType::Plain(value) => value.id,
                    StorageEntryType::Map {
                        hashers: _,
                        key: _,
                        value,
                    } => value.id,
                };
                let metadata_type = get_metadata_type(metadata, type_id);
                let key = get_storage_plain_key(&pallet.name, &storage_entry.name);
                if trace_key.starts_with(&key) {
                    if trace_key == key {
                        log::info!("Found {}.{}", &pallet.name, &storage_entry.name);
                    } else {
                        let _params = trace_key.trim_start_matches(&key);
                        log::info!("Found starts with {}.{}", &pallet.name, &storage_entry.name);
                    }
                    if trace_values.len() > 1 {
                        let mut length = 0;
                        let mut value = "".to_string();
                        if trace_values.first().unwrap().is_some() {
                            let first = trace_values.first().clone().unwrap();
                            let mut bytes: &[u8] = &hex::decode(first.clone().unwrap())?;
                            let compact_length: Compact<u32> = Decode::decode(&mut bytes)?;
                            length = compact_length.0;
                            value = hex::encode(bytes);
                        }
                        for i in 1..trace_values.len() {
                            if let Some(trace_value) = trace_values.get(i).unwrap() {
                                length += 1;
                                value = format!("{value}{trace_value}");
                            }
                        }
                        let mut bytes: &[u8] = &hex::decode(value)?;
                        decode_value(metadata, metadata_type, &mut bytes, false, Some(length), &mut json_buffer)?;
                        println!("");
                    } else {
                        if let Some(Some(value)) = trace_values.first() {
                            if !value.is_empty() {
                                let mut bytes: &[u8] = &hex::decode(value)?;
                                decode_value(metadata, metadata_type, &mut bytes, false, None, &mut json_buffer)?;
                                println!("");
                            }
                        }
                    }
                    is_found = true;
                    break 'outer;
                }
            }
        }
    }
    if !is_found {
        let bytes: &[u8] = &hex::decode(trace_key)?;
        if let Ok(key) = std::str::from_utf8(bytes) {
            match bytes {
                sp_storage::well_known_keys::CHILD_STORAGE_KEY_PREFIX
                | sp_storage::well_known_keys::CODE
                | sp_storage::well_known_keys::DEFAULT_CHILD_STORAGE_KEY_PREFIX
                | sp_storage::well_known_keys::EXTRINSIC_INDEX
                | sp_storage::well_known_keys::HEAP_PAGES
                | sp_storage::well_known_keys::INTRABLOCK_ENTROPY => log::info!("Known key: {key}"),
                _ => {
                    log::error!("Unknown key: {key}");
                }
            }
        } else {
            log::error!("Unknown key: {trace_key}");
        }
    }

    let json_str = json_buffer.join("");
    if !json_str.is_empty() {
        let _json: serde_json::Value = serde_json::from_str(&json_str)?;
    }
    Ok(())
}
