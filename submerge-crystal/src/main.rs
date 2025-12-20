use clap::Parser;
use submerge_base::supervisor::Supervisor;
use submerge_crystal::{args::Args, Crystal};

use tracing::{span, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _tracing_span = span!(Level::TRACE, "Submerge Crystal");
    let args = Args::parse();
    let crystal = Crystal::new(args.clone()).await?;
    Supervisor::new(crystal, args.service.recovery_sleep_seconds)
        .start()
        .await
}
