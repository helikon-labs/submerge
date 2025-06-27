use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use sp_runtime::DigestItem;

#[derive(Serialize, Deserialize, Debug)]
pub struct EventDigest {
    pub logs: Vec<String>,
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

    pub fn get_logs(&self) -> anyhow::Result<Vec<DigestItem>> {
        let mut logs = vec![];
        for log in self.digest.logs.iter() {
            let mut log_bytes: &[u8] = &hex::decode(log.trim_start_matches("0x"))?;
            logs.push(Decode::decode(&mut log_bytes)?);
        }
        Ok(logs)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockWrapper {
    pub block: Block,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub extrinsics: Vec<String>,
}
