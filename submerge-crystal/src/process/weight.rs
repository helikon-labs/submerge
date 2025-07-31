use crate::{
    process::{decode::ValueVisitor, BlockProcessor},
    types::{
        legacy::{LegacyPerDispatchClass, LegacyWeight},
        metadata::util::get_block_weight_type,
        PerDispatchClass,
    },
};
use frame_metadata::RuntimeMetadata;
use parity_scale_codec::Decode;
use serde_json::Value as JsonValue;
use sp_runtime::Weight;

fn from_bytes(bytes: &mut &[u8]) -> anyhow::Result<Option<JsonValue>> {
    let mut copy_bytes: &[u8] = bytes;
    if let Ok(weight) = PerDispatchClass::<Weight>::decode(&mut copy_bytes) {
        return Ok(Some(serde_json::to_value(&weight)?));
    }
    let mut copy_bytes: &[u8] = bytes;
    if let Ok(weight) = PerDispatchClass::<LegacyWeight>::decode(&mut copy_bytes) {
        return Ok(Some(serde_json::to_value(&weight)?));
    }
    let mut copy_bytes: &[u8] = bytes;
    if let Ok(weight) = LegacyPerDispatchClass::<LegacyWeight>::decode(&mut copy_bytes) {
        return Ok(Some(serde_json::to_value(&weight)?));
    }
    anyhow::bail!("Unable to decode block weight.");
}

impl BlockProcessor {
    pub async fn get_block_weight_json_value(
        &self,
        block_hash_hex: &str,
        metadata: &RuntimeMetadata,
    ) -> anyhow::Result<Option<JsonValue>> {
        let weight = if let Some(bytes) = self
            .substrate_client
            .get_block_weight_bytes(block_hash_hex)
            .await?
        {
            let mut bytes: &[u8] = &bytes;
            if let frame_metadata::RuntimeMetadata::V14(metadata_v14) = metadata {
                if let Some(ty) = get_block_weight_type(metadata)? {
                    let visitor = ValueVisitor::new(0, None);
                    let value = scale_decode::visitor::decode_with_visitor(
                        &mut bytes,
                        ty.id,
                        &metadata_v14.types,
                        visitor,
                    )?;
                    Some(value.into())
                } else {
                    from_bytes(&mut bytes)?
                }
            } else {
                from_bytes(&mut bytes)?
            }
        } else {
            None
        };
        Ok(weight)
    }
}
