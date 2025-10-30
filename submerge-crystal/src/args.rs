use clap::Parser;
use submerge_base::args::{HTTPAPIArgs, LoggingArgs, MetricsArgs, PostgreSQLArgs, ServiceArgs};

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[arg(short = 'c', long)]
    /// Name of the chain, or path of the chain specification file
    pub chain: String,

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

    #[clap(flatten)]
    pub logging: LoggingArgs,
}
