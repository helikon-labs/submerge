use serde::Deserialize;
use std::fmt;

const DEFAULT_CONFIG_DIR: &str = "./config";
const DEV_CONFIG_DIR: &str = "../_config";
const DEFAULT_NETWORK: &str = "polkadot";

/// Runtime environment.
#[derive(Clone, Debug)]
pub enum Environment {
    Development,
    Test,
    Production,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Environment::Development => write!(f, "Development"),
            Environment::Test => write!(f, "Test"),
            Environment::Production => write!(f, "Production"),
        }
    }
}

impl From<&str> for Environment {
    fn from(env: &str) -> Self {
        match env.to_lowercase().as_str() {
            "testing" | "test" => Environment::Test,
            "production" | "prod" => Environment::Production,
            "development" | "dev" => Environment::Development,
            _ => panic!("Unknown environment: {env}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CommonConfig {
    pub recovery_retry_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LogConfig {
    pub subvt_level: String,
    pub other_level: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SubstrateConfig {
    pub chain: String,
    pub chain_display: String,
    pub chain_genesis_hash: String,
    pub connection_timeout_seconds: u64,
    pub request_timeout_seconds: u64,
    pub rpc_url: String,
    pub token_decimals: usize,
    pub token_ticker: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetricsConfig {
    pub host: String,
    pub crystal_port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HTTPConfig {
    pub request_timeout_seconds: u64,
    pub service_host: String,
    pub crystal_api_port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PostgreSQLConfig {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub password: String,
    pub pool_max_connections: u32,
    pub connection_timeout_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CrystalConfig {
    pub chainspec_path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub common: CommonConfig,
    pub log: LogConfig,
    pub substrate: SubstrateConfig,
    pub metrics: MetricsConfig,
    pub http: HTTPConfig,
    pub postgres: PostgreSQLConfig,
    pub crystal: CrystalConfig,
}

fn get_config(
    env: &Environment,
    network: &str,
    config_dir: &str,
) -> Result<Config, config::ConfigError> {
    let config = config::Config::builder()
        .set_default("env", env.to_string())?
        .add_source(config::File::with_name(&format!("{config_dir}/base")))
        .add_source(config::File::with_name(&format!(
            "{}/env/{}",
            config_dir,
            env.to_string().to_lowercase()
        )))
        .add_source(config::File::with_name(&format!(
            "{config_dir}/network/{network}",
        )))
        .add_source(config::Environment::with_prefix("submerge").separator("__"))
        .build()?;
    config.try_deserialize()
}

impl Config {
    pub fn test() -> Result<Self, config::ConfigError> {
        let env = Environment::Test;
        get_config(&env, DEFAULT_NETWORK, DEV_CONFIG_DIR)
    }

    fn new() -> Result<Self, config::ConfigError> {
        let env = Environment::from(
            std::env::var("SUBMERGE_ENV")
                .unwrap_or_else(|_| "Production".into())
                .as_str(),
        );
        let network = std::env::var("SUBMERGE_NETWORK").unwrap_or_else(|_| DEFAULT_NETWORK.into());
        let config_dir = if cfg!(debug_assertions) {
            std::env::var("SUBMERGE_CONFIG_DIR").unwrap_or_else(|_| DEV_CONFIG_DIR.into())
        } else {
            std::env::var("SUBMERGE_CONFIG_DIR").unwrap_or_else(|_| DEFAULT_CONFIG_DIR.into())
        };
        get_config(&env, &network, &config_dir)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new().expect("Config can't be loaded.")
    }
}
