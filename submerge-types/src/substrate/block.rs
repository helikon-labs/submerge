use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EventDigest {
    logs: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub digest: EventDigest,
    pub extrinsics_root: String,
    pub number: String,
    pub parent_hash: String,
    pub state_root: String,
}
