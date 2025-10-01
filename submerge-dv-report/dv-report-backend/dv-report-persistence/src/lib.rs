use dv_report_config::Config;
use lazy_static::lazy_static;

pub mod postgres;

lazy_static! {
    static ref CONFIG: Config = Config::default();
}
