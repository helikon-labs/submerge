use submerge_metrics::{
    define_counter, define_counter_vec, define_gauge, define_gauge_vec, define_histogram,
    define_histogram_vec,
};

const METRIC_PREFIX: &str = "submerge_crystal";

define_gauge_vec!(
    METRIC_PREFIX,
    processed_best_block_number,
    "processed_best_block_number",
    "Number of the last processed best block",
    &["worker_id"],
);
define_gauge_vec!(
    METRIC_PREFIX,
    target_best_block_number,
    "target_best_block_number",
    "Number of the target best block",
    &["worker_id"],
);
define_gauge_vec!(
    METRIC_PREFIX,
    processed_finalized_block_number,
    "processed_finalized_block_number",
    "Number of the last processed finalized block",
    &["worker_id"],
);
define_gauge_vec!(
    METRIC_PREFIX,
    target_finalized_block_number,
    "target_finalized_block_number",
    "Number of the target finalized block",
    &["worker_id"],
);

define_histogram_vec!(
    METRIC_PREFIX,
    block_processing_time_ms,
    "block_processing_time_ms",
    "Block processing time in milliseconds",
    &["worker_id"],
    vec![
        10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 250.0, 500.0, 750.0, 1_000.0, 1_500.0,
        2_000.0, 3_000.0, 4_000.0, 5_000.0, 7_500.0, 10_000.0, 15_000.0, 20_000.0,
    ],
);

define_histogram_vec!(
    METRIC_PREFIX,
    block_status_update_time_ms,
    "block_status_update_time_ms",
    "Block status update time in milliseconds",
    &["worker_id"],
    vec![
        10.0, 25.0, 50.0, 75.0, 100.0, 150.0, 200.0, 250.0, 500.0, 750.0, 1_000.0, 1_500.0,
        2_000.0, 3_000.0, 4_000.0, 5_000.0, 7_500.0, 10_000.0, 15_000.0, 20_000.0, 50_000.0,
        100_000.0, 250_000.0,
    ],
);

define_counter!(
    METRIC_PREFIX,
    api_requests_total,
    "api_requests_total",
    "Total API requests",
);

define_gauge!(
    METRIC_PREFIX,
    api_active_connections,
    "api_active_connections",
    "Current active API connections",
);

define_counter_vec!(
    METRIC_PREFIX,
    api_response_status_code_counter,
    "api_response_status_code_counter",
    "The number of response status codes",
    &["status_code"],
);

define_histogram!(
    METRIC_PREFIX,
    api_response_time_ms,
    "api_response_time_ms",
    "API response time in milliseconds",
    vec![
        50.0, 100.0, 250.0, 500.0, 750.0, 1_000.0, 1_500.0, 2_500.0, 5_000.0, 10_000.0, 15_000.0,
        30_000.0,
    ],
);
