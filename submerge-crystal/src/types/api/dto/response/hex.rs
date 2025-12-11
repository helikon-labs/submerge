use serde::Serialize;
use utoipa::ToSchema;

/// Arbitrary hex-encoded bytes. Lowercase, even length, `0x`-prefixed.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(
    value_type = String,
    pattern = "^0x(?:[0-9a-f]{2})+$",
    examples("0xdeadbeef", "0x0123456789abcdef"),
)]
pub struct HexString(pub String);

impl From<&[u8]> for HexString {
    fn from(value: &[u8]) -> Self {
        Self(format!("0x{}", hex::encode(value)))
    }
}

/// 32-byte Blake2b-256, **always** `0x`-prefixed and lowercase in responses.
/// Inputs elsewhere may accept mixed case or missing `0x`; the API normalizes outputs.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(
    value_type = String,
    pattern = "^0x[a-f0-9]{64}$",
    example = "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3",
)]
pub struct Hash256Hex(pub String);

impl From<&[u8]> for Hash256Hex {
    fn from(value: &[u8]) -> Self {
        Self(format!("0x{}", hex::encode(value)))
    }
}

/// 32-byte Substrate account id as lowercase hex, **always** `0x`-prefixed and lowercase in responses.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(
    value_type = String,
    pattern = "^0x[a-f0-9]{64}$",
    example = "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3",
)]
pub struct AccountIdHex(pub String);

/// 20-byte address as lowercase hex, **always** `0x`-prefixed and lowercase in responses.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(
    value_type = String,
    pattern = "^0x[a-f0-9]{40}$",
    example = "0x00112233445566778899aabbccddeeff00112233",
)]
pub struct Address20Hex(pub String);

impl From<&[u8; 20]> for Address20Hex {
    fn from(value: &[u8; 20]) -> Self {
        Self(format!("0x{}", hex::encode(value)))
    }
}

/// 32-byte address as lowercase hex, **always** `0x`-prefixed and lowercase in responses.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(
    value_type = String,
    pattern = "^0x[a-f0-9]{64}$",
    example = "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3",
)]
pub struct Address32Hex(pub String);

impl From<&[u8; 32]> for Address32Hex {
    fn from(value: &[u8; 32]) -> Self {
        Self(format!("0x{}", hex::encode(value)))
    }
}
