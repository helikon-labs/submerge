#[macro_export]
macro_rules! define_gauge {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr,) => {
        pub(crate) fn $name() -> $crate::PrometheusResult<$crate::registry::IntGauge> {
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
        pub(crate) fn $name() -> $crate::PrometheusResult<$crate::registry::IntGaugeVec> {
            static METER: once_cell::sync::OnceCell<$crate::registry::IntGaugeVec> =
                once_cell::sync::OnceCell::new();
            METER
                .get_or_try_init(|| {
                    $crate::registry::register_int_gauge_vec(
                        $prefix,
                        $metric_name,
                        $description,
                        $labels,
                    )
                })
                .cloned()
        }
    };
}

#[macro_export]
macro_rules! define_counter {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr,) => {
        pub(crate) fn $name() -> $crate::PrometheusResult<$crate::registry::IntCounter> {
            static METER: once_cell::sync::OnceCell<$crate::registry::IntCounter> =
                once_cell::sync::OnceCell::new();
            METER
                .get_or_try_init(|| {
                    $crate::registry::register_int_counter($prefix, $metric_name, $description)
                })
                .cloned()
        }
    };
}

#[macro_export]
macro_rules! define_counter_vec {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr, $labels:expr,) => {
        pub(crate) fn $name() -> $crate::PrometheusResult<$crate::registry::IntCounterVec> {
            static METER: once_cell::sync::OnceCell<$crate::registry::IntCounterVec> =
                once_cell::sync::OnceCell::new();
            METER
                .get_or_try_init(|| {
                    $crate::registry::register_int_counter_vec(
                        $prefix,
                        $metric_name,
                        $description,
                        $labels,
                    )
                })
                .cloned()
        }
    };
}

#[macro_export]
macro_rules! define_histogram {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr, $buckets:expr,) => {
        pub(crate) fn $name() -> $crate::PrometheusResult<$crate::registry::Histogram> {
            static METER: once_cell::sync::OnceCell<$crate::registry::Histogram> =
                once_cell::sync::OnceCell::new();
            METER
                .get_or_try_init(|| {
                    $crate::registry::register_histogram(
                        $prefix,
                        $metric_name,
                        $description,
                        $buckets,
                    )
                })
                .cloned()
        }
    };
}

#[macro_export]
macro_rules! define_histogram_vec {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr, $labels:expr, $buckets:expr,) => {
        pub(crate) fn $name() -> $crate::PrometheusResult<$crate::registry::HistogramVec> {
            static METER: once_cell::sync::OnceCell<$crate::registry::HistogramVec> =
                once_cell::sync::OnceCell::new();
            METER
                .get_or_try_init(|| {
                    $crate::registry::register_histogram_vec(
                        $prefix,
                        $metric_name,
                        $description,
                        $labels,
                        $buckets,
                    )
                })
                .cloned()
        }
    };
}
