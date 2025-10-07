use parity_scale_codec::{Decode, Encode};
use serde::Serialize;

use crate::types::substrate::account_id::AccountId;

#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum MultiAddress {
    #[serde(rename = "accountId")]
    Id(AccountId),
    #[serde(rename = "index")]
    Index(#[codec(compact)] u32),
    #[serde(rename = "raw")]
    #[serde(with = "hex_vec")]
    Raw(Vec<u8>),
    #[serde(rename = "address32")]
    #[serde(with = "hex_32")]
    Address32([u8; 32]),
    #[serde(rename = "address20")]
    #[serde(with = "hex_20")]
    Address20([u8; 20]),
}

mod hex_vec {
    use serde::Serializer;
    pub fn serialize<S>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&format!("0x{}", hex::encode(v)))
    }
}

mod hex_20 {
    use serde::Serializer;
    pub fn serialize<S>(v: &[u8; 20], s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&format!("0x{}", hex::encode(v)))
    }
}

mod hex_32 {
    use serde::Serializer;
    pub fn serialize<S>(v: &[u8; 32], s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&format!("0x{}", hex::encode(v)))
    }
}
