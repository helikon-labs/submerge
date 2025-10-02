use clap::Parser;
use log::LevelFilter;
use submerge_base::Supervisor;
use submerge_crystal::{args::Args, Crystal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    submerge_logging::init(LevelFilter::Debug, LevelFilter::Warn);
    let args = match Args::try_parse() {
        Ok(cli) => cli,
        Err(e) => return Err(e.into()),
    };
    let crystal = Crystal::new(args.clone()).await?;
    Supervisor::new(crystal, args.service.recovery_sleep_seconds)
        .start()
        .await
}
