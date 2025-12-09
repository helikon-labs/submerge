use serde::Serialize;
use utoipa::ToSchema;

pub mod block;
pub mod call;
pub mod event;
pub mod extrinsic;
pub mod genesis;
pub mod metadata;
pub mod multi_address;
pub mod pagination;
pub mod trace;

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

/// 32-byte Substrate account id, **always** `0x`-prefixed and lowercase in responses.
/// Inputs elsewhere may accept mixed case or missing `0x`; the API normalizes outputs.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[schema(
    value_type = String,
    pattern = "^0x[a-f0-9]{64}$",
    example = "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3",
)]
pub struct AccountIdHex(pub String);

fn block_weight_schema() -> utoipa::openapi::Object {
    use utoipa::openapi::ObjectBuilder;

    ObjectBuilder::new()
        .schema_type(utoipa::openapi::schema::Type::Object)
        .examples([Some(serde_json::json!({
            "normal": {
                "refTime": "0",
                "proofSize": "0"
            },
            "mandatory": {
                "refTime": "361766342408",
                "proofSize": "592668"
            },
            "operational": {
                "refTime": "0",
                "proofSize": "0"
            },
        }))])
        .description(Some(
            "Block weight in JSON format. Schema depends on runtime metadata.".to_string(),
        ))
        .build()
}
