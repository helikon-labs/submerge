use clap::Parser;
use submerge_base::args::{HTTPAPIArgs, LoggingArgs, MetricsArgs, PostgreSQLArgs, ServiceArgs};

#[derive(Parser, Clone, Debug)]
#[command(version)]
pub struct Args {
    #[clap(flatten)]
    pub postgres: PostgreSQLArgs,

    #[clap(flatten)]
    pub api: HTTPAPIArgs,

    #[clap(flatten)]
    pub metrics: MetricsArgs,

    #[clap(flatten)]
    pub service: ServiceArgs,

    #[clap(flatten)]
    pub logging: LoggingArgs,

    /// Name of the chain, or path of the chain specification file
    #[arg(short = 'c', long)]
    pub chain: String,

    /// URL of the legacy decode API for block content pre-metadata-v14
    #[arg(long)]
    pub legacy_decode_api_url: Option<String>,

    /// Default RPC node URL for workers
    #[arg(long)]
    pub default_rpc_url: String,
}
