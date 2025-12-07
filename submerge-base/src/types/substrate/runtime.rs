use parity_scale_codec::Decode;
use serde::Deserialize;
use submerge_util::substrate::storage::decode_hex_string;

#[derive(Deserialize, Default, Decode)]
#[serde(rename_all = "camelCase")]
pub struct LastRuntimeUpgradeInfo {
    pub spec_version: u32,
    pub spec_name: String,
}

impl From<frame_system::LastRuntimeUpgradeInfo> for LastRuntimeUpgradeInfo {
    fn from(upgrade: frame_system::LastRuntimeUpgradeInfo) -> Self {
        Self {
            spec_version: upgrade.spec_version.0,
            spec_name: upgrade.spec_name.to_string(),
        }
    }
}

impl LastRuntimeUpgradeInfo {
    pub fn from_substrate_hex_string(hex_string: String) -> anyhow::Result<Self> {
        Ok(decode_hex_string::<frame_system::LastRuntimeUpgradeInfo>(&hex_string)?.into())
    }
}
