use std::str::FromStr as _;

use anyhow::Context as _;
use rustc_hash::FxHashMap as HashMap;
use serde::{Deserialize, Serialize};

use crate::types::substrate::chain::Chain;

#[derive(rust_embed::Embed)]
#[folder = "../_chainspecs"]
#[include = "*.json"]
struct Chainspecs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Chainspec {
    pub name: String,
    pub id: String,
    pub genesis: Genesis,
    pub properties: ChainProperties,
}

impl Chainspec {
    pub fn from_chain_name_or_file_path(s: &str) -> anyhow::Result<Self> {
        if let Ok(chain) = Chain::from_str(s) {
            let chainspec_path = chain.get_chainspec_path();
            if let Some(chainspec_file) = Chainspecs::get(&chainspec_path) {
                let chainspec_json_str = std::str::from_utf8(chainspec_file.data.as_ref())?;
                serde_json::from_str(chainspec_json_str).map_err(|error| {
                    anyhow::anyhow!("🔴 Failed to parse {chain} chainspec JSON: {error:?}")
                })
            } else {
                anyhow::bail!(format!(
                    "Chainspec for {chain} not found at path: {chainspec_path}"
                ));
            }
        } else {
            let chainspec_json_str = std::fs::read_to_string(s).context(format!(
                "🔴 {s} is not a supported chain name or a valid chainspec file path."
            ))?;
            serde_json::from_str(&chainspec_json_str)
                .context(format!("🔴 Failed to parse chainspec JSON at: {s}"))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum TokenDecimals {
    Single(u64),
    Multiple(Vec<u64>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum TokenSymbol {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Genesis {
    pub raw: RawGenesis,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RawGenesis {
    pub top: HashMap<String, String>,
}

const fn default_ss58_format() -> u16 {
    42
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChainProperties {
    #[serde(default = "default_ss58_format")]
    pub ss58_format: u16,
    pub token_decimals: TokenDecimals,
    pub token_symbol: TokenSymbol,
}

#[cfg(test)]
mod tests {
    use convert_case::{Case, Casing};
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_get_chainspec_by_chain_name() {
        let chainspec = Chainspec::from_chain_name_or_file_path("coretime-westend")
            .expect("Failed to get the chainspec for Coretime Westend.");
        tracing::info!("Chain name: {}", chainspec.name);
        assert_eq!(chainspec.name, "Westend Coretime");
        assert_eq!(chainspec.id, "coretime-westend");
        assert!(chainspec
            .genesis
            .raw
            .top
            .contains_key("0x0d715f2646c8f85767b5d2764bb2782604a74d81251e398fd8a0a4d55023bb3f"));
    }

    #[test]
    fn test_get_all_chainspecs_by_chain_name() {
        for chain in Chain::iter() {
            let _ =
                Chainspec::from_chain_name_or_file_path(&chain.to_string().to_case(Case::Kebab))
                    .expect("Failed to get the chainspec for chain.");
        }
    }

    #[test]
    fn test_get_chainspec_by_file_path() {
        let file_path = "../_chainspecs/westend/sys/coretime-westend.json";
        let chainspec = Chainspec::from_chain_name_or_file_path(file_path)
            .expect("Failed to get the chainspec for Coretime Westend.");
        tracing::info!("Chain name: {}", chainspec.name);
        assert_eq!(chainspec.name, "Westend Coretime");
        assert_eq!(chainspec.id, "coretime-westend");
        assert!(chainspec
            .genesis
            .raw
            .top
            .contains_key("0x0d715f2646c8f85767b5d2764bb2782604a74d81251e398fd8a0a4d55023bb3f"));
    }
}
