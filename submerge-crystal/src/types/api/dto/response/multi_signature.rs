use serde::Serialize;
use submerge_base::types::substrate::multi_signature::MultiSignature;
use utoipa::ToSchema;

use crate::types::api::dto::response::hex::SignatureHexString;

/// Multi-signature ECDSA type.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MultiSignatureECDSAType {
    Ecdsa,
}

/// ECDSA signature.
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = MultiSignatureECDSA)]
pub struct MultiSignatureEcdsaDTO {
    /// Must be `ecdsa`.
    #[serde(rename = "type")]
    pub r#type: MultiSignatureECDSAType,

    /// ECDSA signature hex.
    pub value: SignatureHexString,
}

/// Multi-signature Ed25519 type.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MultiSignatureEd25519Type {
    Ed25519,
}

/// Ed25519 signature.
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = MultiSignatureEd25519)]
pub struct MultiSignatureEd25519DTO {
    /// Must be `ed25519`.
    #[serde(rename = "type")]
    pub r#type: MultiSignatureEd25519Type,

    /// Ed25519 signature hex.
    pub value: SignatureHexString,
}

/// Multi-signature Sr25519 type.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MultiSignatureSr25519Type {
    Sr25519,
}

/// Sr25519 signature.
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = MultiSignatureSr25519)]
pub struct MultiSignatureSr25519DTO {
    /// Must be `sr25519`.
    #[serde(rename = "type")]
    pub r#type: MultiSignatureSr25519Type,

    /// Sr25519 signature hex.
    pub value: SignatureHexString,
}

/// Discriminated union for the Substrate multi-signature type.
/// The `type` tag selects the variant; `value` carries the payload.
#[derive(Debug, Serialize, ToSchema)]
#[serde(untagged)]
#[schema(
    as = MultiSignature,
    discriminator(property_name = "type", mapping(
        ("ecdsa" = "#/components/schemas/MultiSignatureECDSA"),
        ("ed25519" = "#/components/schemas/MultiSignatureEd25519"),
        ("sr25519" = "#/components/schemas/MultiSignatureSr25519"),
    )),
    example = json!({
        "type": "sr25519",
        "value": "0xabababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababababab",
    })
)]
pub enum MultiSignatureDTO {
    Ecdsa(MultiSignatureEcdsaDTO),
    Ed25519(MultiSignatureEd25519DTO),
    Sr25519(MultiSignatureSr25519DTO),
}

impl From<&MultiSignature> for MultiSignatureDTO {
    fn from(value: &MultiSignature) -> Self {
        match value {
            MultiSignature::Ecdsa(bytes) => Self::Ecdsa(MultiSignatureEcdsaDTO {
                r#type: MultiSignatureECDSAType::Ecdsa,
                value: SignatureHexString(hex::encode(bytes)),
            }),
            MultiSignature::Ed25519(bytes) => Self::Ed25519(MultiSignatureEd25519DTO {
                r#type: MultiSignatureEd25519Type::Ed25519,
                value: SignatureHexString(hex::encode(bytes)),
            }),
            MultiSignature::Sr25519(bytes) => Self::Sr25519(MultiSignatureSr25519DTO {
                r#type: MultiSignatureSr25519Type::Sr25519,
                value: SignatureHexString(hex::encode(bytes)),
            }),
        }
    }
}
