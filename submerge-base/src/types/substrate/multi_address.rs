use std::str::FromStr;

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
    #[serde(with = "hex_serde")]
    Raw(Vec<u8>),
    #[serde(rename = "address32")]
    #[serde(with = "hex_serde")]
    Address32([u8; 32]),
    #[serde(rename = "address20")]
    #[serde(with = "hex_serde")]
    Address20([u8; 20]),
}

mod hex_serde {
    use serde::Serializer;

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(bytes)))
    }
}

impl FromStr for MultiAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // try account id
        if let Ok(account_id) = AccountId::from_str(s) {
            return Ok(MultiAddress::Id(account_id));
        }
        // try hex decoding
        let trimmed = s.trim_start_matches("0x");
        let bytes = hex::decode(trimmed)?;
        // try fixed-size arrays without cloning
        match bytes.len() {
            32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(MultiAddress::Address32(arr))
            }
            20 => {
                let mut arr = [0u8; 20];
                arr.copy_from_slice(&bytes);
                Ok(MultiAddress::Address20(arr))
            }
            _ => Ok(MultiAddress::Raw(bytes)),
        }
    }
}
