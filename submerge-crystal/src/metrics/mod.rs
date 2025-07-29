use once_cell::sync::Lazy;
use submerge_metrics::registry::{Histogram, IntCounter, IntCounterVec, IntGauge};

const METRIC_PREFIX: &str = "submerge_crystal";

pub fn processed_best_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        submerge_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "processed_best_block_number",
            "Number of the last processed block",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn target_best_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        submerge_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "target_best_block_number",
            "Number of the target finalized block",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn processed_finalized_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        submerge_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "processed_finalized_block_number",
            "Number of the last processed block",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn target_finalized_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        submerge_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "target_finalized_block_number",
            "Number of the target finalized block",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn block_processing_time_ms() -> Histogram {
    static METER: Lazy<Histogram> = Lazy::new(|| {
        submerge_metrics::registry::register_histogram(
            METRIC_PREFIX,
            "block_processing_time_ms",
            "Block processing time in milliseconds",
            vec![
                10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 250.0, 500.0, 750.0, 1_000.0, 1_500.0,
                2_000.0, 3_000.0, 4_000.0, 5_000.0, 7_500.0, 10_000.0, 15_000.0, 20_000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}

pub fn block_status_update_time_ms() -> Histogram {
    static METER: Lazy<Histogram> = Lazy::new(|| {
        submerge_metrics::registry::register_histogram(
            METRIC_PREFIX,
            "block_status_update_time_ms",
            "Block status update time in milliseconds",
            vec![
                10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 250.0, 500.0, 750.0, 1_000.0, 1_500.0,
                2_000.0, 3_000.0, 4_000.0, 5_000.0, 7_500.0, 10_000.0, 15_000.0, 20_000.0,
                50_000.0, 100_000.0, 250_000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}

pub(crate) fn request_counter() -> IntCounter {
    static METER: Lazy<IntCounter> = Lazy::new(|| {
        submerge_metrics::registry::register_int_counter(
            METRIC_PREFIX,
            "request_count",
            "The total number of requests made to the API",
        )
        .unwrap()
    });
    METER.clone()
}

pub(crate) fn open_connection_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        submerge_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "connection_count",
            "Number of API connections currently open",
        )
        .unwrap()
    });
    METER.clone()
}

pub(crate) fn response_time_ms() -> Histogram {
    static METER: Lazy<Histogram> = Lazy::new(|| {
        submerge_metrics::registry::register_histogram(
            METRIC_PREFIX,
            "response_time_ms",
            "Response time in milliseconds",
            vec![
                50.0, 100.0, 250.0, 500.0, 750.0, 1_000.0, 1_500.0, 2_500.0, 5_000.0, 10_000.0,
                15_000.0, 30_000.0,
            ],
        )
        .unwrap()
    });
    METER.clone()
}

pub(crate) fn response_status_code_counter(status_code: &str) -> IntCounter {
    static METER: Lazy<IntCounterVec> = Lazy::new(|| {
        submerge_metrics::registry::register_int_counter_vec(
            METRIC_PREFIX,
            "response_status_code_count",
            "The number of response status codes",
            &["status_code"],
        )
        .unwrap()
    });
    METER.with_label_values(&[status_code])
}
