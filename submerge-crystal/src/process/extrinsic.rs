use frame_metadata::{v16::StorageHasher, RuntimeMetadata};
use parity_scale_codec::{Compact, Decode, Encode, Input};
use serde_json::Value as JsonValue;
use sqlx::{Postgres, Transaction};
use submerge_base::types::substrate::{
    block::BlockHeader, block_trace::BlockTrace, Balance, MultiAddress, Signature,
};
use submerge_util::substrate::storage::{self, get_storage_plain_key};

use crate::{
    persistence::CrystalPostgreSQLStorage,
    process::{decode::JsonValueVisitor, BlockProcessor},
};

impl BlockProcessor {
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub async fn process_extrinsics(
        &self,
        block_hash_hex: &str,
        block_header: &BlockHeader,
        block_timestamp: u64,
        spec_version: u32,
        metadata_version: u32,
        metadata: &RuntimeMetadata,
        trace: &BlockTrace,
        is_finalized: bool,
        extrinsic_count: u32,
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let block_hash = hex::decode(block_hash_hex)?;
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
            let (pallet_name, call_name) = match metadata {
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
                    tx,
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
        Ok(())
    }
}
