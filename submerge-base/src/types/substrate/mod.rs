use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_json::Value as JSONValue;
use sp_runtime::AccountId32;

pub mod account_id;
pub mod block;
pub mod block_trace;
pub mod chainspec;
pub mod runtime;

pub const BLOCK_HASH_HEX_LENGTH: usize = 64;
pub type Balance = u128;

#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq, Serialize)]
pub enum MultiAddress {
    Id(AccountId32),
    Index(#[codec(compact)] u32),
    Raw(Vec<u8>),
    Address32([u8; 32]),
    Address20([u8; 20]),
}

#[derive(Clone, Debug, Serialize)]
pub struct Signature {
    pub signer: MultiAddress,
    pub signature: sp_runtime::MultiSignature,
    pub extra: Option<JSONValue>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SystemHealth {
    pub peers: u32,
    pub is_syncing: bool,
    pub should_have_peers: bool,
}
