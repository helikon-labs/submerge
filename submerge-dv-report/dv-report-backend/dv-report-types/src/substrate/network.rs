use serde::{Deserialize, Serialize};
use sp_core::crypto::Ss58AddressFormat;
use sqlx::FromRow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub id: u32,
    pub chain: String,
    pub display: String,
    pub rpc_url: String,
    pub token_ticker: String,
    pub token_decimals: usize,
    pub token_format_decimal_points: usize,
    pub ss58_prefix: u16,
}

impl Network {
    pub fn polkadot() -> Self {
        Network {
            id: 1,
            chain: "polkadot".to_string(),
            display: "Polkadot".to_string(),
            rpc_url: "wss://rpc.helikon.io:443/polkadot".to_string(),
            token_ticker: "DOT".to_string(),
            token_decimals: 10,
            token_format_decimal_points: 4,
            ss58_prefix: 0,
        }
    }

    pub fn kusama() -> Self {
        Network {
            id: 2,
            chain: "kusama".to_string(),
            display: "Kusama".to_string(),
            rpc_url: "wss://rpc.helikon.io:443/kusama".to_string(),
            token_ticker: "KSM".to_string(),
            token_decimals: 12,
            token_format_decimal_points: 4,
            ss58_prefix: 2,
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
}

#[derive(Debug)]
pub struct ParseChainError(String);

impl Display for ParseChainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseChainError {}

impl FromStr for Network {
    type Err = ParseChainError;

    /// Get network from string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kusama" | "ksm" => Ok(Self::kusama()),
            "polkadot" | "dot" => Ok(Self::polkadot()),
            _ => Err(ParseChainError(format!("Unknown chain: {s}"))),
        }
    }
}

impl Network {
    pub fn from_id(id: u32) -> Self {
        match id {
            2 => Self::kusama(),
            _ => Self::polkadot(),
        }
    }

    pub fn sp_core_set_default_ss58_version(&self) {
        sp_core::crypto::set_default_ss58_version(Ss58AddressFormat::from(self.ss58_prefix));
    }
}

#[derive(Debug, FromRow)]
pub struct NetworkRow {
    pub id: i32,
    pub hash: String,
    pub chain: String,
    pub display: String,
    pub ss58_prefix: i32,
    pub token_ticker: String,
    pub token_decimal_count: i32,
}
