use once_cell::sync::Lazy;
use prometheus::opts;
pub use prometheus::{
    core::Collector, proto, Counter, CounterVec, Error, Gauge, GaugeVec, Histogram, HistogramOpts,
    HistogramTimer, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Registry,
};
use std::sync::Arc;

static DEFAULT_REGISTRY: Lazy<Arc<Registry>> = Lazy::new(|| Arc::new(Registry::new()));

pub(crate) fn get_default_registry() -> Arc<Registry> {
    Arc::clone(&DEFAULT_REGISTRY)
}

fn register<C: Collector + 'static>(c: C) -> prometheus::Result<()> {
    let registry = get_default_registry();
    registry.register(Box::new(c))
}

fn register_safe<C: Collector + Clone + 'static>(c: C) -> prometheus::Result<C> {
    match register(c.clone()) {
        Ok(_) => Ok(c),
        Err(prometheus::Error::AlreadyReg) => Ok(c),
        Err(e) => Err(e),
    }
}

pub fn register_gauge(prefix: &str, name: &str, help: &str) -> prometheus::Result<Gauge> {
    let gauge = Gauge::new(format!("{prefix}::{name}"), help)?;
    register_safe(gauge.clone())?;
    Ok(gauge)
}

#[allow(clippy::disallowed_types)]
pub fn register_gauge_vec(
    prefix: &str,
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<GaugeVec> {
    let gauge = GaugeVec::new(opts!(format!("{prefix}_{name}"), help), label_names)?;
    register_safe(gauge.clone())?;
    Ok(gauge)
}

#[allow(clippy::disallowed_types)]
pub fn register_int_gauge_vec(
    prefix: &str,
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<IntGaugeVec> {
    let gauge = IntGaugeVec::new(opts!(format!("{prefix}_{name}"), help), label_names)?;
    register_safe(gauge.clone())?;
    Ok(gauge)
}

pub fn register_counter(prefix: &str, name: &str, help: &str) -> prometheus::Result<Counter> {
    let counter = Counter::new(format!("{prefix}_{name}"), help)?;
    register_safe(counter.clone())?;
    Ok(counter)
}

#[allow(clippy::disallowed_types)]
pub fn register_counter_vec(
    prefix: &str,
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<CounterVec> {
    let counter = CounterVec::new(opts!(format!("{prefix}_{name}"), help), label_names)?;
    register_safe(counter.clone())?;
    Ok(counter)
}

pub fn register_int_counter(
    prefix: &str,
    name: &str,
    help: &str,
) -> prometheus::Result<IntCounter> {
    let gauge = IntCounter::new(format!("{prefix}_{name}"), help)?;
    register_safe(gauge.clone())?;
    Ok(gauge)
}

#[allow(clippy::disallowed_types)]
pub fn register_int_counter_vec(
    prefix: &str,
    name: &str,
    help: &str,
    label_names: &[&str],
) -> prometheus::Result<IntCounterVec> {
    let gauge = IntCounterVec::new(opts!(format!("{prefix}_{name}"), help), label_names)?;
    register_safe(gauge.clone())?;
    Ok(gauge)
}

pub fn register_int_gauge(prefix: &str, name: &str, help: &str) -> prometheus::Result<IntGauge> {
    let gauge = IntGauge::new(format!("{prefix}_{name}"), help)?;
    register_safe(gauge.clone())?;
    Ok(gauge)
}

pub fn register_histogram(
    prefix: &str,
    name: &str,
    help: &str,
    buckets: Vec<f64>,
) -> prometheus::Result<Histogram> {
    let histogram = Histogram::with_opts(
        HistogramOpts::new(format!("{prefix}_{name}"), help).buckets(buckets),
    )?;
    register_safe(histogram.clone())?;
    Ok(histogram)
}

pub fn register_histogram_vec(
    prefix: &str,
    name: &str,
    help: &str,
    label_names: &[&str],
    buckets: Vec<f64>,
) -> prometheus::Result<HistogramVec> {
    let gauge = HistogramVec::new(
        HistogramOpts::new(format!("{prefix}_{name}"), help).buckets(buckets),
        label_names,
    )?;
    register_safe(gauge.clone())?;
    Ok(gauge)
}
