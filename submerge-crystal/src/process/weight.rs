use crate::{
    process::{decode::ValueVisitor, BlockProcessor},
    types::metadata::util::get_block_weight_type,
};
use frame_metadata::RuntimeMetadata;
use serde_json::Value as JSONValue;
use submerge_base::types::substrate::block_trace::BlockTrace;
use submerge_util::substrate::storage::get_storage_plain_key;

impl BlockProcessor {
    async fn decode_block_weight(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        metadata: &RuntimeMetadata,
        bytes: &mut &[u8],
    ) -> anyhow::Result<JSONValue> {
        if let frame_metadata::RuntimeMetadata::V14(metadata_v14) = metadata {
            let Some(ty) = get_block_weight_type(metadata_v14)? else {
                anyhow::bail!("System.BlockWeight type not found in metadata.");
            };
            let visitor = ValueVisitor::new(0, None);
            let value = scale_decode::visitor::decode_with_visitor(
                bytes,
                ty.id,
                &metadata_v14.types,
                visitor,
            )?;
            Ok(value.into())
        } else {
            let legacy_decode_api_client = if let Some(client) = &self.legacy_decode_api_client {
                client
            } else {
                anyhow::bail!(
                    "Legacy decode API client is not set. legacy_decode_api_url parameter not set."
                );
            };
            let block_weight = legacy_decode_api_client
                .decode_block_weight(block_hash, spec_version, bytes)
                .await?;
            Ok(block_weight)
        }
    }

    pub async fn get_block_weight_from_rpc(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        metadata: &RuntimeMetadata,
    ) -> anyhow::Result<Option<JSONValue>> {
        if let Some(bytes) = self
            .substrate_client
            .get_block_weight_bytes(&hex::encode(block_hash))
            .await?
        {
            let mut bytes: &[u8] = &bytes;
            let block_weight = self
                .decode_block_weight(block_hash, spec_version, metadata, &mut bytes)
                .await?;
            Ok(Some(block_weight))
        } else {
            Ok(None)
        }
    }

    pub async fn get_block_weight_from_trace(
        &self,
        block_hash: &[u8],
        spec_version: u32,
        metadata: &RuntimeMetadata,
        trace: &BlockTrace,
    ) -> anyhow::Result<Option<JSONValue>> {
        let block_weight_key = get_storage_plain_key("System", "BlockWeight");
        let mut maybe_trace_index_value: Option<(usize, String)> = None;
        for (trace_index, trace) in trace.events.iter().enumerate() {
            let trace_data = &trace.data_wrapper.data;
            if trace_data.key != block_weight_key
                || trace_data.value.to_lowercase() == "none"
                || trace_data.value.is_empty()
            {
                continue;
            }
            let value = trace_data
                .value
                .trim_start_matches("Some(")
                .trim_end_matches(")");
            if let Some(trace_index_value) = &maybe_trace_index_value {
                if trace_index_value.0 < trace_index {
                    maybe_trace_index_value = Some((trace_index, value.to_string()));
                }
            } else {
                maybe_trace_index_value = Some((trace_index, value.to_string()));
            }
        }
        let Some(trace_index_value) = maybe_trace_index_value else {
            return Ok(None);
        };
        let mut bytes: &[u8] = &hex::decode(&trace_index_value.1)?;
        Ok(Some(
            self.decode_block_weight(block_hash, spec_version, metadata, &mut bytes)
                .await?,
        ))
    }
}
