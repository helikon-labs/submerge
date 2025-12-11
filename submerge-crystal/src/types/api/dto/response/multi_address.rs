use serde::Serialize;
use submerge_base::types::substrate::multi_address::MultiAddress;
use utoipa::ToSchema;

use super::hex::{AccountIdHex, Address20Hex, Address32Hex, HexString};

/// Multi-address 32-byte account id type.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum MultiAddressAccountIdType {
    AccountId,
}

/// 32-byte account id.
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = MultiAddressAccountId)]
pub struct MultiAddressAccountIdDTO {
    /// Must be `accountId`.
    #[serde(rename = "type")]
    pub r#type: MultiAddressAccountIdType,

    /// Account id as 32-byte hex.
    pub value: AccountIdHex,
}

/// Multi-address account index type.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum MultiAddressIndexType {
    Index,
}

/// Account index.
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = MultiAddressAccountIndex)]
pub struct MultiAddressIndexDTO {
    /// Must be `index`.
    #[serde(rename = "type")]
    pub r#type: MultiAddressIndexType,

    /// Account index value.
    #[schema(example = 83)]
    pub value: u32,
}

/// Multi-address raw account address type.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum MultiAddressRawType {
    Raw,
}

/// Raw account address.
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = MultiAddressRaw)]
pub struct MultiAddressRawDTO {
    /// Must be `raw`.
    #[serde(rename = "type")]
    pub r#type: MultiAddressRawType,
    pub value: HexString,
}

/// Multi-address 20-byte address type.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum MultiAddress20Type {
    Address20,
}

/// 20-byte address.
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = MultiAddress20)]
pub struct MultiAddress20DTO {
    /// Must be `address20`.
    #[serde(rename = "type")]
    pub r#type: MultiAddress20Type,
    pub value: Address20Hex,
}

/// Multi-address 32-byte address type.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum MultiAddress32Type {
    Address32,
}

/// 32-byte address.
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = MultiAddress32)]
pub struct MultiAddress32DTO {
    /// Must be `address32`.
    #[serde(rename = "type")]
    pub r#type: MultiAddress32Type,
    pub value: Address32Hex,
}

/// Discriminated union for the Substrate multiaddress type.
/// The `type` tag selects the variant; `value` carries the payload.
#[derive(Debug, Serialize, ToSchema)]
#[serde(untagged)]
#[schema(
    as = MultiAddress,
    discriminator(property_name = "type", mapping(
        ("accountId" = "#/components/schemas/MultiAddressAccountId"),
        ("index" = "#/components/schemas/MultiAddressIndex"),
        ("raw" = "#/components/schemas/MultiAddressRaw"),
        ("address20" = "#/components/schemas/MultiAddress20"),
        ("address32" = "#/components/schemas/MultiAddress32"),
    )),
    example = json!({
        "type": "accountId",
        "value": "0x008d8404893c7b4b80f397605cc96e61fec3c89676c8c2794a2a7d281d678b1a",
    })
)]
pub enum MultiAddressDTO {
    AccountId(MultiAddressAccountIdDTO),
    Index(MultiAddressIndexDTO),
    Raw(MultiAddressRawDTO),
    Address20(MultiAddress20DTO),
    Address32(MultiAddress32DTO),
}

impl From<&MultiAddress> for MultiAddressDTO {
    fn from(value: &MultiAddress) -> Self {
        match value {
            MultiAddress::Id(account_id) => Self::AccountId(MultiAddressAccountIdDTO {
                r#type: MultiAddressAccountIdType::AccountId,
                value: AccountIdHex(format!("0x{account_id}")),
            }),
            MultiAddress::Index(index) => Self::Index(MultiAddressIndexDTO {
                r#type: MultiAddressIndexType::Index,
                value: *index,
            }),
            MultiAddress::Raw(bytes) => Self::Raw(MultiAddressRawDTO {
                r#type: MultiAddressRawType::Raw,
                value: bytes.as_slice().into(),
            }),
            MultiAddress::Address20(bytes) => Self::Address20(MultiAddress20DTO {
                r#type: MultiAddress20Type::Address20,
                value: bytes.into(),
            }),
            MultiAddress::Address32(bytes) => Self::Address32(MultiAddress32DTO {
                r#type: MultiAddress32Type::Address32,
                value: bytes.into(),
            }),
        }
    }
}
