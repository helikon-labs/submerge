use clap::Parser;

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

    #[arg(long, default_value_t = 256)]
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
    #[arg(long, default_value = "0.0.0.0")]
    /// HTTP API listen host
    pub api_host: String,

    #[arg(long, default_value_t = 3030)]
    /// HTTP API listen port
    pub api_port: u16,

    /// API rate limit window (e.g. 5 requests per 1000 milliseconds) - default 1000 ms (1 sec)
    #[arg(long, default_value_t = 1000)]
    pub api_rate_limit_window_ms: u64,

    /// API rate limit request count (e.g. 5 requests per 1000 milliseconds) - default 5 requests
    #[arg(long, default_value_t = 5)]
    pub api_rate_limit_request_count: u32,
}

#[derive(Parser, Clone, Debug)]
pub struct MetricsArgs {
    #[arg(long, default_value = "0.0.0.0")]
    /// Prometheus server listen host
    pub metrics_host: String,

    #[arg(long, default_value_t = 13030)]
    /// Prometheus server listen port
    pub metrics_port: u16,
}

#[derive(Parser, Clone, Debug)]
pub struct LoggingArgs {
    /// Log level for the native Submerge crates
    #[arg(long, default_value = "debug")]
    pub native_log_level: String,

    /// Log level for all crates external to Submerge
    #[arg(long, default_value = "warn")]
    pub external_log_level: String,
}
