#![warn(clippy::disallowed_types)]
use dv_report_config::Config;
use env_logger::{Builder, Env, Target, WriteStyle};
use std::str::FromStr;

pub fn init(config: &Config) {
    let other_modules_log_level = log::LevelFilter::from_str(config.log.other_level.as_str())
        .expect("Cannot read log level configuration for outside modules.");
    let log_level = log::LevelFilter::from_str(config.log.dv_report_level.as_str())
        .expect("Cannot read log level configuration for DV Report modules.");
    let mut builder = Builder::from_env(Env::default());
    builder.target(Target::Stdout);
    builder.filter(None, other_modules_log_level);
    builder.filter(Some("dv_report_api_service"), log_level);
    builder.filter(Some("dv_report_config"), log_level);
    builder.filter(Some("dv_report_indexer"), log_level);
    builder.filter(Some("dv_report_logging"), log_level);
    builder.filter(Some("dv_report_metrics"), log_level);
    builder.filter(Some("dv_report_persistence"), log_level);
    builder.filter(Some("dv_report_repository"), log_level);
    builder.filter(Some("dv_report_service"), log_level);
    builder.filter(Some("dv_report_subsquare_client"), log_level);
    builder.filter(Some("dv_report_substrate_client"), log_level);
    builder.filter(Some("dv_report_types"), log_level);
    builder.write_style(WriteStyle::Always);
    builder.init();
}
