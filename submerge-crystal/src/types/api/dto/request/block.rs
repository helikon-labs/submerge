use std::str::FromStr as _;

use serde::{Deserialize, Serialize};
use submerge_base::types::substrate::multi_address::MultiAddress;
use utoipa::IntoParams;

use crate::types::api::error::APIError;

use crate::types::BlockStatus;

pub(crate) enum BlockReference {
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
#[derive(Debug, Deserialize, IntoParams, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BlockQuery {
    /// Opaque cursor for block pagination - returned in the endpoint response.
    /// This parameter is mutually exclusive with all other parameters,
    /// and will return bad request if any other parameter is set.
    #[param(required = false, nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Number of blocks per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Filter results by block status. If not specified, all blocks are returned.
    #[param(required = false, nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<BlockStatus>,
    /// Filter blocks by minimum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 1534287)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_block_number: Option<u64>,
    /// Filter blocks by maximum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 2825701)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block_number: Option<u64>,
    /// Filter blocks by minimum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_block_timestamp: Option<u64>,
    /// Filter blocks by maximum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block_timestamp: Option<u64>,
    /// Filter blocks by minimum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_spec_version: Option<u32>,
    /// Filter blocks by maximum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

impl BlockQuery {
    pub(crate) fn get_author_multi_address(&self) -> anyhow::Result<Option<MultiAddress>> {
        let author = if let Some(author) = &self.author {
            Some(MultiAddress::from_str(author)?)
        } else {
            None
        };
        Ok(author)
    }

    pub(crate) fn validate_next_cursor_mutually_exclusive(&self) -> Result<(), APIError> {
        if self.next_cursor.is_some() {
            let mut other_fields = Vec::new();
            if self.page_size.is_some() {
                other_fields.push("page_size");
            }
            if self.status.is_some() {
                other_fields.push("status");
            }
            if self.min_block_number.is_some() {
                other_fields.push("min_block_number");
            }
            if self.max_block_number.is_some() {
                other_fields.push("max_block_number");
            }
            if self.min_block_timestamp.is_some() {
                other_fields.push("min_block_timestamp");
            }
            if self.max_block_timestamp.is_some() {
                other_fields.push("max_block_timestamp");
            }
            if self.min_spec_version.is_some() {
                other_fields.push("min_spec_version");
            }
            if self.max_spec_version.is_some() {
                other_fields.push("max_spec_version");
            }
            if self.author.is_some() {
                other_fields.push("author");
            }
            if !other_fields.is_empty() {
                return Err(APIError::BadRequest(
                    format!(
                        "No other parameter should not be set when next_cursor is set. Please remove these parameters and try again: {}",
                        other_fields.join(", ")
                    )
                ));
            }
        }
        Ok(())
    }
}
