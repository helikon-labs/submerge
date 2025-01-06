#![warn(clippy::disallowed_types)]
use env_logger::{Builder, Env, Target, WriteStyle};
pub use log::LevelFilter;

pub fn init(log_level: LevelFilter, other_log_level: LevelFilter) {
    let mut builder = Builder::from_env(Env::default());
    builder.target(Target::Stdout);
    builder.filter(None, other_log_level);
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
