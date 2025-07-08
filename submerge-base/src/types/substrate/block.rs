use parity_scale_codec::Decode;
use serde::{Deserialize, Serialize};
use sp_consensus_babe::digests::PreDigest;
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
            let digest_item = Decode::decode(&mut log_bytes)?;
            logs.push(digest_item);
        }
        Ok(logs)
    }

    fn authority_index_from_log_bytes(
        consensus_engine: &str,
        mut bytes: &[u8],
    ) -> anyhow::Result<u32> {
        match consensus_engine.to_lowercase().as_str() {
            "babe" => {
                let digest: PreDigest = Decode::decode(&mut bytes).unwrap();
                let authority_index = match digest {
                    PreDigest::Primary(digest) => digest.authority_index,
                    PreDigest::SecondaryPlain(digest) => digest.authority_index,
                    PreDigest::SecondaryVRF(digest) => digest.authority_index,
                };
                Ok(authority_index)
            }
            "aura" => {
                let authority_index = Decode::decode(&mut bytes)?;
                Ok(authority_index)
            }
            _ => {
                anyhow::bail!(
                    "Consensus engine [{}] does not support direct author extraction from logs.",
                    consensus_engine
                );
            }
        }
    }

    pub fn get_validator_index(&self) -> anyhow::Result<u32> {
        let mut validator_index: Option<u32> = None;
        for log_string in &self.digest.logs {
            let log_hex_string = log_string.trim_start_matches("0x");
            let mut log_bytes: &[u8] = &hex::decode(log_hex_string)?;
            let digest_item: DigestItem = Decode::decode(&mut log_bytes)?;
            match digest_item {
                DigestItem::PreRuntime(consensus_engine_id, bytes) => {
                    let consensus_engine = std::str::from_utf8(&consensus_engine_id)?;
                    validator_index = Some(BlockHeader::authority_index_from_log_bytes(
                        consensus_engine,
                        &bytes,
                    )?);
                }
                DigestItem::Consensus(consensus_engine_id, bytes) => {
                    if validator_index.is_none() {
                        let consensus_engine = std::str::from_utf8(&consensus_engine_id)?;
                        validator_index = Some(BlockHeader::authority_index_from_log_bytes(
                            consensus_engine,
                            &bytes,
                        )?);
                    }
                }
                DigestItem::Seal(consensus_engine_id, bytes) => {
                    if validator_index.is_none() {
                        let consensus_engine = std::str::from_utf8(&consensus_engine_id)?;
                        validator_index = Some(BlockHeader::authority_index_from_log_bytes(
                            consensus_engine,
                            &bytes,
                        )?);
                    }
                }
                DigestItem::RuntimeEnvironmentUpdated => {
                    log::warn!(
                        "Log type: RuntimeEnvironmentUpdated. Cannot get author validator index."
                    );
                }
                DigestItem::Other(_) => {
                    log::warn!("Unknown log type. Cannot get author validator index.");
                }
            }
            if let Some(validator_index) = validator_index {
                return Ok(validator_index);
            }
        }
        anyhow::bail!("Author validator index not found.");
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
