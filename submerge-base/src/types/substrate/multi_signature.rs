use parity_scale_codec::{Decode, Encode};
use serde::Serialize;
use submerge_util::serde::hex_serde;

#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "lowercase")]
pub enum MultiSignature {
    #[serde(with = "hex_serde")]
    Ed25519(sp_core::ed25519::Signature),
    #[serde(with = "hex_serde")]
    Sr25519(sp_core::sr25519::Signature),
    #[serde(with = "hex_serde")]
    Ecdsa(sp_core::ecdsa::Signature),
    #[serde(with = "hex_serde")]
    Eth(sp_core::ecdsa::KeccakSignature),
}

impl From<sp_runtime::MultiSignature> for MultiSignature {
    fn from(value: sp_runtime::MultiSignature) -> Self {
        match value {
            sp_runtime::MultiSignature::Ed25519(bytes) => Self::Ed25519(bytes),
            sp_runtime::MultiSignature::Sr25519(bytes) => Self::Sr25519(bytes),
            sp_runtime::MultiSignature::Ecdsa(bytes) => Self::Ecdsa(bytes),
            sp_runtime::MultiSignature::Eth(bytes) => Self::Eth(bytes),
        }
    }
}
