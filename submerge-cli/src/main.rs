use anyhow::anyhow;
use clap::{Parser, Subcommand};
use log::LevelFilter;
use submerge_base::Supervisor;
use submerge_crystal::Crystal;

#[derive(Parser)]
#[command(about, version, propagate_version = true)]
#[allow(clippy::upper_case_acronyms)]
/// Submerge command-line interface
pub struct CLI {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
/// Run Submerge Crystal
pub enum Command {
    Crystal(submerge_crystal::args::Args),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    submerge_logging::init(LevelFilter::Debug, LevelFilter::Warn);
    let cli = match CLI::try_parse() {
        Ok(cli) => cli,
        Err(e) => return Err(e.into()),
    };
    if let Some(command) = &cli.command {
        match command {
            Command::Crystal(args) => {
                let crystal = Crystal::new(args.clone());
                Supervisor::new(crystal, args.service.recovery_sleep_seconds) // 10 second retry delay
                    .start()
                    .await
            }
        }
    } else {
        Err(anyhow!("No subcommand provided. Launch interactive CLI."))
    }
}
