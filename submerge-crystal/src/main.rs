use clap::Parser;
use submerge_base::supervisor::Supervisor;
use submerge_crystal::{args::Args, Crystal};

use tracing::{span, Level};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _tracing_span = span!(Level::TRACE, "Submerge Crystal");
    let args = match Args::try_parse() {
        Ok(cli) => cli,
        Err(e) => return Err(e.into()),
    };
    let crystal = Crystal::new(args.clone()).await?;
    Supervisor::new(crystal, args.service.recovery_sleep_seconds)
        .start()
        .await
}
