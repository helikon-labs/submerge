use clap::Parser;
use submerge_base::args::{HTTPAPIArgs, MetricsArgs, PostgreSQLArgs, RPCArgs, ServiceArgs};

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[arg(short = 's', long)]
    /// Start indexing from this block (inclusive)
    pub start_block: Option<u64>,

    #[arg(short = 'e', long)]
    /// End indexing at this block (inclusive)
    pub end_block: Option<u64>,

    #[arg(short = 'c', long)]
    /// Path of the chain specification file
    pub chainspec_path: String,

    #[arg(long)]
    /// Do not start the Prometheus server
    pub no_metrics: bool,

    #[arg(long)]
    /// Do not start the HTTP REST API
    pub no_api: bool,

    #[clap(flatten)]
    pub rpc: RPCArgs,

    #[clap(flatten)]
    pub postgres: PostgreSQLArgs,

    #[clap(flatten)]
    pub api: HTTPAPIArgs,

    #[clap(flatten)]
    pub metrics: MetricsArgs,

    #[clap(flatten)]
    pub service: ServiceArgs,
}
