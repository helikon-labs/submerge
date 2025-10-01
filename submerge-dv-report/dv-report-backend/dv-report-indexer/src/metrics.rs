use dv_report_metrics::registry::IntGauge;
use once_cell::sync::Lazy;

const METRIC_PREFIX: &str = "dv_report_indexer";

pub fn indexed_finalized_block_number() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        dv_report_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "indexed_finalized_block_number",
            "Number of the last processed block",
        )
        .unwrap()
    });
    METER.clone()
}

pub fn imported_comment_count() -> IntGauge {
    static METER: Lazy<IntGauge> = Lazy::new(|| {
        dv_report_metrics::registry::register_int_gauge(
            METRIC_PREFIX,
            "imported_comment_count",
            "Number of imported comments from Subsquare",
        )
        .unwrap()
    });
    METER.clone()
}
