use clap::Parser;
use submerge_cli::args::{HTTPAPIArgs, MetricsArgs, PostgreSQLArgs, RPCArgs, ServiceArgs};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(super) struct Args {
    #[arg(short = 's', long)]
    pub start_block: Option<u64>,

    #[arg(short = 'e', long)]
    pub end_block: Option<u64>,

    #[arg(short = 'p', long)]
    pub chainspec_path: String,

    #[arg(long)]
    pub no_metrics: bool,

    #[arg(long)]
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
