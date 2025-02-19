use clap::{Parser, Subcommand};
use log::LevelFilter;
use submerge_base::BaseService;
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
async fn main() {
    let cli = match CLI::try_parse() {
        Ok(cli) => cli,
        Err(e) => return println!("{e}"),
    };
    if let Some(command) = &cli.command {
        match command {
            Command::Crystal(args) => {
                let crystal = Crystal::new(args.clone());
                crystal.start().await;
            }
        };
    } else {
        submerge_logging::init(LevelFilter::Debug, LevelFilter::Warn);
        log::info!("No subcommand provided. Launch interactive CLI.");
    }
}
