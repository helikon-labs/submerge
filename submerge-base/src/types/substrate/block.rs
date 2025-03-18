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

impl BlockHeader {
    /// Number from the hex string.
    pub fn get_number(&self) -> anyhow::Result<u64> {
        let number = u64::from_str_radix(self.number.trim_start_matches("0x"), 16)?;
        Ok(number)
    }
}
