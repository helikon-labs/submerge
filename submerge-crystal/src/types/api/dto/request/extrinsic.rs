use std::str::FromStr as _;

use serde::Deserialize;
use submerge_base::types::substrate::multi_address::MultiAddress;
use utoipa::IntoParams;

/// Query parameters for fetching and filtering extrinsics.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub struct ExtrinsicQuery {
    /// Extrinsic list page number to retrieve. 1-indexed.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        default = 1,
        example = 1
    )]
    pub page: Option<u32>,
    /// Number of extrinsics per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    pub page_size: Option<u32>,
    /// Filter extrinsics by minimum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 1534287)]
    pub min_block_number: Option<u64>,
    /// Filter extrinsics by maximum block number.
    #[param(required = false, nullable = false, minimum = 0, example = 2825701)]
    pub max_block_number: Option<u64>,
    /// Filter extrinsics by minimum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    pub min_block_timestamp: Option<u64>,
    /// Filter extrinsics by maximum block timestamp. In milliseconds.
    #[param(
        required = false,
        nullable = false,
        minimum = 0,
        example = 1755773684012u64
    )]
    pub max_block_timestamp: Option<u64>,
    /// Filter extrinsics by minimum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    pub min_spec_version: Option<u32>,
    /// Filter extrinsics by maximum runtime spec version.
    #[param(required = false, nullable = false, minimum = 0, example = 1090)]
    pub max_spec_version: Option<u32>,
    /// Whether to include only signed/unsigned extrinsics.
    #[param(required = false, nullable = false, example = true)]
    pub is_signed: Option<bool>,
    /// Filter extrinsics by signer. Either of the following:
    /// - Signer's Substrate SS58 address string (e.g. `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY`).
    /// - Signer's Substrate account id encoded as a hex string (e.g. `0x008d8404893c7b4b80f397605cc96e61fec3c89676c8c2794a2a7d281d678b1a`).
    /// - Signer's address encoded as a hex string with optional `0x` prefix (e.g. `0x008d8404893c7b4b80f397605cc96e61fec3c89676c8c2794a2a7d281d678b1a`).
    #[param(
        required = false,
        nullable = false,
        pattern = "^(?:[1-9A-HJ-NP-Za-km-z]{47,48}|(?:0x)?[0-9a-fA-F]{1-256})$",
        example = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    )]
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

/// Query parameters for fetching and filtering extrinsics within a block.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
pub struct BlockExtrinsicQuery {
    /// Block call list page number to retrieve. 1-indexed.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        default = 1,
        example = 1
    )]
    pub page: Option<u32>,
    /// Number of block calls per page to be returned.
    #[param(
        required = false,
        nullable = false,
        minimum = 1,
        maximum = 100,
        default = 25,
        example = 50
    )]
    pub page_size: Option<u32>,
    /// Whether to include only signed/unsigned extrinsics within the block.
    #[param(required = false, nullable = false, example = true)]
    pub is_signed: Option<bool>,
    /// Filter block extrinsics by signer. Either of the following:
    /// - Signer's Substrate SS58 address string (e.g. `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY`).
    /// - Signer's Substrate account id encoded as a hex string (e.g. `0x008d8404893c7b4b80f397605cc96e61fec3c89676c8c2794a2a7d281d678b1a`).
    /// - Signer's address encoded as a hex string with optional `0x` prefix (e.g. `0x008d8404893c7b4b80f397605cc96e61fec3c89676c8c2794a2a7d281d678b1a`).
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
