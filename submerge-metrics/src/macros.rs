#[macro_export]
macro_rules! define_gauge {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr,) => {
        pub fn $name() -> $crate::PrometheusResult<$crate::registry::IntGauge> {
            static METER: once_cell::sync::OnceCell<$crate::registry::IntGauge> =
                once_cell::sync::OnceCell::new();
            METER
                .get_or_try_init(|| {
                    $crate::registry::register_int_gauge($prefix, $metric_name, $description)
                })
                .cloned()
        }
    };
}

#[macro_export]
macro_rules! define_gauge_vec {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr, $labels:expr,) => {
        pub fn $name() -> $crate::registry::IntGaugeVec {
            static METER: once_cell::sync::Lazy<$crate::registry::IntGaugeVec> =
                once_cell::sync::Lazy::new(|| {
                    $crate::registry::register_int_gauge_vec(
                        $prefix,
                        $metric_name,
                        $description,
                        $labels,
                    )
                    .unwrap()
                });
            METER.clone()
        }
    };
}

#[macro_export]
macro_rules! define_counter {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr,) => {
        pub fn $name() -> $crate::registry::IntCounter {
            static METER: once_cell::sync::Lazy<$crate::registry::IntCounter> =
                once_cell::sync::Lazy::new(|| {
                    $crate::registry::register_int_counter($prefix, $metric_name, $description)
                        .unwrap()
                });
            METER.clone()
        }
    };
}

#[macro_export]
macro_rules! define_counter_vec {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr, $labels:expr,) => {
        pub fn $name() -> $crate::registry::IntCounterVec {
            static METER: once_cell::sync::Lazy<$crate::registry::IntCounterVec> =
                once_cell::sync::Lazy::new(|| {
                    $crate::registry::register_int_counter_vec(
                        $prefix,
                        $metric_name,
                        $description,
                        $labels,
                    )
                    .unwrap()
                });
            METER.clone()
        }
    };
}

#[macro_export]
macro_rules! define_histogram {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr, $buckets:expr,) => {
        pub fn $name() -> $crate::registry::Histogram {
            static METER: once_cell::sync::Lazy<$crate::registry::Histogram> =
                once_cell::sync::Lazy::new(|| {
                    $crate::registry::register_histogram(
                        $prefix,
                        $metric_name,
                        $description,
                        $buckets,
                    )
                    .unwrap()
                });
            METER.clone()
        }
    };
}

#[macro_export]
macro_rules! define_histogram_vec {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr, $labels:expr, $buckets:expr,) => {
        pub fn $name() -> $crate::registry::HistogramVec {
            static METER: once_cell::sync::Lazy<$crate::registry::HistogramVec> =
                once_cell::sync::Lazy::new(|| {
                    $crate::registry::register_histogram_vec(
                        $prefix,
                        $metric_name,
                        $description,
                        $labels,
                        $buckets,
                    )
                    .unwrap()
                });
            METER.clone()
        }
    };
}
