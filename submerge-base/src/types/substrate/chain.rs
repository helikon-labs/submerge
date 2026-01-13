use std::str::FromStr;

use convert_case::{Case, Casing};
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;

#[derive(Copy, Clone, Debug, Display, EnumIter)]
pub enum Chain {
    // Polkadot and parachains
    Polkadot,
    AssetHubPolkadot,
    BridgeHubPolkadot,
    CollectivesPolkadot,
    CoretimePolkadot,
    PeoplePolkadot,
    Acala,
    Astar,
    BifrostPolkadot,
    Centrifuge,
    Hydration,
    HyperbridgeNexus,
    Moonbeam,
    Mythos,
    // Kusama and parachains
    Kusama,
    AssetHubKusama,
    BridgeHubKusama,
    CoretimeKusama,
    PeopleKusama,
    BifrostKusama,
    Moonriver,
    // Westend and parachains
    Westend,
    CoretimeWestend,
    // Paseo and parachains
    Paseo,
    AssetHubPaseo,
    BridgeHubPaseo,
    CollectivesPaseo,
    CoretimePaseo,
    PeoplePaseo,
}

impl FromStr for Chain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for chain in Chain::iter() {
            if s == chain.to_string().to_case(Case::Kebab).as_str() {
                return Ok(chain);
            }
        }
        Err(format!("Unsupported chain: {s}"))
    }
}

impl Chain {
    pub fn get_chainspec_path(&self) -> String {
        let dir = match self {
            Chain::Polkadot => "polkadot",
            Chain::AssetHubPolkadot
            | Chain::BridgeHubPolkadot
            | Chain::CollectivesPolkadot
            | Chain::CoretimePolkadot
            | Chain::PeoplePolkadot => "polkadot/sys",
            Chain::Acala
            | Chain::Astar
            | Chain::BifrostPolkadot
            | Chain::Centrifuge
            | Chain::Hydration
            | Chain::HyperbridgeNexus
            | Chain::Moonbeam
            | Chain::Mythos => "polkadot/para",
            Chain::Kusama => "kusama",
            Chain::AssetHubKusama
            | Chain::BridgeHubKusama
            | Chain::CoretimeKusama
            | Chain::PeopleKusama => "kusama/sys",
            Chain::BifrostKusama | Chain::Moonriver => "kusama/para",
            Chain::Westend => "westend",
            Chain::CoretimeWestend => "westend/sys",
            Chain::Paseo => "paseo",
            Chain::AssetHubPaseo
            | Chain::BridgeHubPaseo
            | Chain::CollectivesPaseo
            | Chain::CoretimePaseo
            | Chain::PeoplePaseo => "paseo/sys",
        };
        format!("{dir}/{}.json", self.to_string().to_case(Case::Kebab))
    }
}
