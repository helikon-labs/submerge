pub use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct RPCArgs {
    #[arg(short = 'r', long, default_value = "wss://rpc.helikon.io/polkadot")]
    pub rpc_url: String,

    #[arg(long, default_value = "20")]
    pub rpc_connection_timeout_secs: u64,

    #[arg(long, default_value = "20")]
    pub rpc_request_timeout_secs: u64,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct PostgreSQLArgs {
    #[arg(long, default_value = "localhost")]
    pub postgres_host: String,

    #[arg(long, default_value_t = 5432)]
    pub postgres_port: u16,

    #[arg(long, default_value = "submerge")]
    pub postgres_username: String,

    #[arg(long, default_value = "submerge")]
    pub postgres_password: String,

    #[arg(long, default_value = "submerge")]
    pub postgres_db_name: String,

    #[arg(long, default_value_t = 5)]
    pub postgres_connection_timeout_secs: u64,

    #[arg(long, default_value_t = 100)]
    pub postgres_pool_max_connections: u32,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ServiceArgs {
    #[arg(long, default_value_t = 5)]
    pub recovery_sleep_seconds: u64,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct HTTPAPIArgs {
    #[arg(long, default_value = "localhost")]
    pub api_host: String,

    #[arg(long, default_value_t = 3030)]
    pub api_port: u16,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct MetricsArgs {
    #[arg(long, default_value = "localhost")]
    pub metrics_host: String,

    #[arg(long, default_value_t = 13030)]
    pub metrics_port: u16,
}
