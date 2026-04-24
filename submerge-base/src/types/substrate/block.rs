use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sp_consensus_babe::digests::PreDigest;
use sp_runtime::DigestItem;

#[derive(Clone, Serialize, Deserialize, Debug, Encode)]
pub struct EventDigest {
    pub logs: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Encode)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub digest: EventDigest,
    pub extrinsics_root: String,
    pub number: String,
    pub parent_hash: String,
    pub state_root: String,
}

impl BlockHeader {
    /// Number from the hexadecimal string.
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
        consensus_engine_id: [u8; 4],
        mut bytes: &[u8],
    ) -> anyhow::Result<Option<u32>> {
        let consensus_engine = std::str::from_utf8(&consensus_engine_id)?;
        match consensus_engine.to_lowercase().as_str() {
            "aura" => {
                let authority_index = Decode::decode(&mut bytes)?;
                Ok(Some(authority_index))
            }
            "babe" => {
                let digest: PreDigest = Decode::decode(&mut bytes)?;
                let authority_index = match digest {
                    PreDigest::Primary(digest) => digest.authority_index,
                    PreDigest::SecondaryPlain(digest) => digest.authority_index,
                    PreDigest::SecondaryVRF(digest) => digest.authority_index,
                };
                Ok(Some(authority_index))
            }
            "cmls" => Ok(None),
            _ => {
                anyhow::bail!(
                    "Consensus engine [{}] does not support direct author extraction from logs.",
                    consensus_engine
                );
            }
        }
    }

    pub fn get_validator_index(&self) -> anyhow::Result<Option<u32>> {
        for log_string in &self.digest.logs {
            let log_hex_string = log_string.trim_start_matches("0x");
            let log_bytes_vec = hex::decode(log_hex_string)?;
            let mut log_bytes: &[u8] = &log_bytes_vec;
            let digest_item: DigestItem = Decode::decode(&mut log_bytes)?;
            match digest_item {
                DigestItem::PreRuntime(consensus_engine_id, bytes) => {
                    if let Some(authority_index) =
                        BlockHeader::authority_index_from_log_bytes(consensus_engine_id, &bytes)?
                    {
                        return Ok(Some(authority_index));
                    }
                }
                DigestItem::Consensus(consensus_engine_id, bytes) => {
                    if let Some(authority_index) =
                        BlockHeader::authority_index_from_log_bytes(consensus_engine_id, &bytes)?
                    {
                        return Ok(Some(authority_index));
                    }
                }
                DigestItem::Seal(_, _) => {
                    // skipped: Seal does not contain validator index
                }
                DigestItem::RuntimeEnvironmentUpdated => {
                    tracing::warn!(
                        "Log type: RuntimeEnvironmentUpdated. Cannot get author validator index."
                    );
                }
                DigestItem::Other(_) => {
                    tracing::warn!("Unknown log type. Cannot get author validator index.");
                }
            }
        }
        Ok(None)
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

impl Block {
    pub fn get_raw_extrinsic_bytes(&self) -> anyhow::Result<Vec<Vec<u8>>> {
        let mut raw_extrinsics = Vec::new();
        for extrinsic in self.extrinsics.iter() {
            raw_extrinsics.push(hex::decode(extrinsic.trim_start_matches("0x"))?);
        }
        Ok(raw_extrinsics)
    }
}

#[derive(Clone, Debug)]
pub struct DecodedBlockHeader {
    pub parent_hash: Vec<u8>,
    pub state_root: Vec<u8>,
    pub extrinsic_root: Vec<u8>,
    pub number: u64,
}

impl TryFrom<&BlockHeader> for DecodedBlockHeader {
    type Error = anyhow::Error;

    fn try_from(header: &BlockHeader) -> Result<Self, anyhow::Error> {
        Ok(Self {
            parent_hash: hex::decode(&header.parent_hash)?,
            state_root: hex::decode(&header.state_root)?,
            extrinsic_root: hex::decode(&header.extrinsics_root)?,
            number: header.get_number()?,
        })
    }
}
