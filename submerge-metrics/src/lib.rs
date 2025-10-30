#![warn(clippy::disallowed_types)]
pub use prometheus::Result as PrometheusResult;

pub mod macros;
pub mod registry;
pub mod server;

pub fn use_metric<T, F>(result: prometheus::Result<T>, f: F)
where
    F: FnOnce(&T),
{
    match result {
        Ok(metric) => f(&metric),
        Err(e) => tracing::warn!("Metric error: {}", e),
    }
}
