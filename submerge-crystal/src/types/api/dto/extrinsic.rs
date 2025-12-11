use std::str::FromStr as _;

use serde::Deserialize;
use submerge_base::types::substrate::multi_address::MultiAddress;

use crate::types::api::dto::pagination::PaginationQuery;

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
