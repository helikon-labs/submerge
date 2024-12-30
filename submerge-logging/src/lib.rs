#![warn(clippy::disallowed_types)]
use env_logger::{Builder, Env, Target, WriteStyle};
use std::str::FromStr;

pub fn init(config: &submerge_config::Config) {
    let other_modules_log_level = log::LevelFilter::from_str(config.log.other_level.as_str())
        .expect("Cannot read log level configuration for outside modules.");
    let log_level = log::LevelFilter::from_str(config.log.subvt_level.as_str())
        .expect("Cannot read log level configuration for SubVT modules.");
    let mut builder = Builder::from_env(Env::default());
    builder.target(Target::Stdout);
    builder.filter(None, other_modules_log_level);
    builder.filter(Some("submerge_api"), log_level);
    builder.filter(Some("submerge_auth"), log_level);
    builder.filter(Some("submerge_base"), log_level);
    builder.filter(Some("submerge_bloom"), log_level);
    builder.filter(Some("submerge_cli"), log_level);
    builder.filter(Some("submerge_config"), log_level);
    builder.filter(Some("submerge_cortex"), log_level);
    builder.filter(Some("submerge_crystal"), log_level);
    builder.filter(Some("submerge_fractal"), log_level);
    builder.filter(Some("submerge_logging"), log_level);
    builder.filter(Some("submerge_metrics"), log_level);
    builder.filter(Some("submerge_persistence"), log_level);
    builder.filter(Some("submerge_reflex"), log_level);
    builder.filter(Some("submerge_sentinel"), log_level);
    builder.filter(Some("submerge_substrate_client"), log_level);
    builder.filter(Some("submerge_types"), log_level);
    builder.filter(Some("submerge_web"), log_level);
    builder.write_style(WriteStyle::Always);
    builder.init();
}
