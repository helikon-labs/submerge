use clap::Parser;

#[derive(Parser, Clone, Debug)]
pub struct RPCArgs {
    #[arg(long, default_value = "wss://rpc.helikon.io/polkadot")]
    /// JSON-RPC server WS URL
    pub rpc_url: String,

    #[arg(long, default_value = "20")]
    /// JSON-RPC connection timeout in seconds
    pub rpc_connection_timeout_secs: u64,

    #[arg(long, default_value = "20")]
    /// JSON-RPC request timeout in seconds
    pub rpc_request_timeout_secs: u64,
}

#[derive(Parser, Clone, Debug)]
pub struct PostgreSQLArgs {
    #[arg(long, default_value = "localhost")]
    /// PostgreSQL server host
    pub postgres_host: String,

    #[arg(long, default_value_t = 5432)]
    /// PostgreSQL server port
    pub postgres_port: u16,

    #[arg(long, default_value = "submerge")]
    /// PostgreSQL server username
    pub postgres_username: String,

    #[arg(long, default_value = "submerge")]
    /// PostgreSQL server password
    pub postgres_password: String,

    #[arg(long, default_value = "submerge")]
    /// PostgreSQL database name
    pub postgres_db_name: String,

    #[arg(long, default_value_t = 5)]
    /// PostgreSQL connection timeout in seconds
    pub postgres_connection_timeout_secs: u64,

    #[arg(long, default_value_t = 100)]
    /// PostgreSQL connection pool max connections
    pub postgres_pool_max_connections: u32,
}

#[derive(Parser, Clone, Debug)]
pub struct ServiceArgs {
    #[arg(long, default_value_t = 5)]
    /// The service will sleep this long in seconds before a restart
    pub recovery_sleep_seconds: u64,
}

#[derive(Parser, Clone, Debug)]
pub struct HTTPAPIArgs {
    #[arg(long, default_value = "localhost")]
    /// HTTP API listen host
    pub api_host: String,

    #[arg(long, default_value_t = 3030)]
    /// HTTP API listen port
    pub api_port: u16,
}

#[derive(Parser, Clone, Debug)]
pub struct MetricsArgs {
    #[arg(long, default_value = "localhost")]
    /// Prometheus server listen host
    pub metrics_host: String,

    #[arg(long, default_value_t = 13030)]
    /// Prometheus server listen port
    pub metrics_port: u16,
}
