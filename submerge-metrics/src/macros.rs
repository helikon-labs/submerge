#[macro_export]
macro_rules! define_gauge {
    ($prefix:expr, $name:ident, $metric_name:expr, $description:expr,) => {
        pub fn $name() -> $crate::registry::IntGauge {
            static METER: once_cell::sync::Lazy<$crate::registry::IntGauge> =
                once_cell::sync::Lazy::new(|| {
                    $crate::registry::register_int_gauge($prefix, $metric_name, $description)
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
