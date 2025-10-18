use clap::Parser;
use log::LevelFilter;
use submerge_base::Supervisor;
use submerge_crystal::{args::Args, Crystal};

use tracing::{event, info, span, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    submerge_logging::init(LevelFilter::Debug, LevelFilter::Warn);
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    {
        let span = span!(Level::TRACE, "Test tracing span.");
        let _enter = span.enter();
        event!(parent: &span, Level::INFO, "Test tracing event.");
        info!(parent: &span, "Test tracing info log.");
    }
    let args = match Args::try_parse() {
        Ok(cli) => cli,
        Err(e) => return Err(e.into()),
    };
    let crystal = Crystal::new(args.clone()).await?;
    Supervisor::new(crystal, args.service.recovery_sleep_seconds)
        .start()
        .await
}
