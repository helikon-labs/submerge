use std::str::FromStr as _;

use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;
use submerge_base::types::substrate::multi_address::MultiAddress;

use crate::types::{api::dto::pagination::PaginationQuery, persistence::ExtrinsicRow, BlockStatus};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtrinsicQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub min_block_number: Option<u64>,
    pub max_block_number: Option<u64>,
    pub min_block_timestamp: Option<u64>,
    pub max_block_timestamp: Option<u64>,
    pub min_spec_version: Option<u32>,
    pub max_spec_version: Option<u32>,
    pub is_signed: Option<bool>,
    pub signer: Option<String>,
}

impl ExtrinsicQuery {
    pub fn get_signer_multi_address(&self) -> anyhow::Result<Option<MultiAddress>> {
        let signer = if let Some(signer) = &self.signer {
            Some(MultiAddress::from_str(signer)?)
        } else {
            None
        };
        Ok(signer)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockExtrinsicQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub is_signed: Option<bool>,
    pub signer: Option<String>,
}

impl BlockExtrinsicQuery {
    pub fn get_signer_multi_address(&self) -> anyhow::Result<Option<MultiAddress>> {
        let signer = if let Some(signer) = &self.signer {
            Some(MultiAddress::from_str(signer)?)
        } else {
            None
        };
        Ok(signer)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtrinsicDTO {
    pub block_hash: String,
    pub block_number: u64,
    pub block_timestamp: Option<u64>,
    pub spec_version: u32,
    pub block_status: BlockStatus,
    pub trace_index: Option<u32>,
    pub hash: String,
    pub index: u32,
    pub version: u32,
    pub signer: Option<MultiAddress>,
    pub signature: Option<String>,
    pub is_successful: bool,
    pub extra: Option<JSONValue>,
}

impl TryFrom<&ExtrinsicRow> for ExtrinsicDTO {
    type Error = anyhow::Error;

    fn try_from(row: &ExtrinsicRow) -> Result<Self, Self::Error> {
        Ok(Self {
            block_hash: format!("0x{}", hex::encode(&row.block_hash)),
            block_number: row.block_number as u64,
            block_timestamp: row.block_timestamp.map(|timestamp| timestamp as u64),
            spec_version: row.spec_version as u32,
            block_status: row.block_status,
            trace_index: row.trace_index.map(|i| i as u32),
            hash: format!("0x{}", hex::encode(row.hash)),
            index: row.index as u32,
            version: row.version as u32,
            signer: if let Some(bytes) = &row.signer_multi_address {
                let mut bytes: &[u8] = bytes;
                Some(MultiAddress::decode(&mut bytes)?)
            } else {
                None
            },
            signature: row
                .signature
                .as_ref()
                .map(|signature| format!("0x{}", hex::encode(signature))),
            is_successful: row.is_successful,
            extra: row.extra.clone(),
        })
    }
}
