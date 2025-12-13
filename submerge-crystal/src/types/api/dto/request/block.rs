use std::str::FromStr as _;

use serde::Deserialize;
use submerge_base::types::substrate::multi_address::MultiAddress;
use utoipa::IntoParams;

use crate::types::BlockStatus;

pub enum BlockReference {
    Hash(Vec<u8>),
    Number(u64),
}

impl TryFrom<&str> for BlockReference {
    type Error = String;

    fn try_from(reference: &str) -> Result<Self, Self::Error> {
        if let Ok(number) = reference.parse::<u64>() {
            Ok(Self::Number(number))
        } else if let Ok(hash) = hex::decode(reference.trim_start_matches("0x")) {
            Ok(Self::Hash(hash))
        } else {
            Err("Invalid block reference. It should be either a block number (integer ≥ 0), or a block hash in hex (with or without 0x prefix, case-insensitive).".to_string())
        }
    }
}

/// Query parameters for fetching and filtering blocks.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub struct BlockQuery {
    /// Block list page number to retrieve. 1-indexed.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        default = 1,
        example = 1
    )]
    pub page: Option<u32>,
    /// Number of blocks per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    pub page_size: Option<u32>,
    /// Filter results by block status. If not specified, all blocks are returned.
    #[param(required = false, nullable = false)]
    pub status: Option<BlockStatus>,
    /// Filter blocks by minimum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 1534287)]
    pub min_block_number: Option<u64>,
    /// Filter blocks by maximum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 2825701)]
    pub max_block_number: Option<u64>,
    /// Filter blocks by minimum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    pub min_block_timestamp: Option<u64>,
    /// Filter blocks by maximum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    pub max_block_timestamp: Option<u64>,
    /// Filter blocks by minimum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    pub min_spec_version: Option<u32>,
    /// Filter blocks by maximum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    pub max_spec_version: Option<u32>,
    /// Filter results by block author. Either of the following:
    /// - Author's Substrate SS58 address string (e.g. `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY`).
    /// - Author's Substrate account id encoded as a hexadecimal string (e.g. `0x008d8404893c7b4b80f397605cc96e61fec3c89676c8c2794a2a7d281d678b1a`).
    /// - Author's address encoded as a hexadecimal string with optional `0x` prefix (e.g. `0x008d8404893c7b4b80f397605cc96e61fec3c89676c8c2794a2a7d281d678b1a`).
    #[param(
        required = false,
        nullable = false,
        pattern = "^(?:[1-9A-HJ-NP-Za-km-z]{47,48}|(?:0x)?[0-9a-fA-F]{1-256})$",
        example = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    )]
    pub author: Option<String>,
}

impl BlockQuery {
    pub fn get_author_multi_address(&self) -> anyhow::Result<Option<MultiAddress>> {
        let author = if let Some(author) = &self.author {
            Some(MultiAddress::from_str(author)?)
        } else {
            None
        };
        Ok(author)
    }
}
