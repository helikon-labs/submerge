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

    #[arg(long, default_value = "false")]
    /// Check each block in the range to see if it was ingested before, rather than relying on
    /// range checks by block number
    pub scan: bool,

    #[arg(long, default_value = "false")]
    /// Errors will always be persisted, and block processing will stop if this switch is on
    pub stop_on_error: bool,

    #[arg(long, default_value = "false")]
    /// Whether to ignore and skip trace records for blocks
    pub skip_traces: bool,

    #[arg(long)]
    pub legacy_decode_api_url: Option<String>,

    #[arg(long, default_value = "false")]
    /// Do not start the Prometheus server
    pub no_metrics: bool,

    #[arg(long, default_value = "false")]
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
