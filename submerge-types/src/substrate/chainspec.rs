use rustc_hash::FxHashMap as HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Chainspec {
    pub name: String,
    pub id: String,
    pub genesis: Genesis,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use submerge_logging::LevelFilter;

    #[test]
    fn test_parse_chainspec_json() {
        submerge_logging::init(LevelFilter::Debug, LevelFilter::Warn);
        let file_path = "../_chainspecs/coretime-westend.json";
        log::info!("Parse chainspec file: {}", &file_path);
        let json_data = fs::read_to_string(file_path).expect("Failed to read JSON file.");
        let parsed: Chainspec = serde_json::from_str(&json_data).expect("Failed to parse JSON.");
        log::info!("Chain name: {}", parsed.name);
        assert_eq!(parsed.name, "Westend Coretime");
        assert_eq!(parsed.id, "coretime-westend");
        assert!(parsed
            .genesis
            .raw
            .top
            .contains_key("0x0d715f2646c8f85767b5d2764bb2782604a74d81251e398fd8a0a4d55023bb3f"));
    }
}
