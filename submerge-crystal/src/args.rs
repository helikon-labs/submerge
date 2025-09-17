use clap::Parser;
use submerge_base::args::{HTTPAPIArgs, MetricsArgs, PostgreSQLArgs, ServiceArgs};

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[arg(short = 'c', long)]
    /// Path of the chain specification file
    pub chainspec_path: String,

    #[arg(long)]
    pub legacy_decode_api_url: Option<String>,

    #[clap(flatten)]
    pub postgres: PostgreSQLArgs,

    #[clap(flatten)]
    pub api: HTTPAPIArgs,

    #[clap(flatten)]
    pub metrics: MetricsArgs,

    #[clap(flatten)]
    pub service: ServiceArgs,
}
