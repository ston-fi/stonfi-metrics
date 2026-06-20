use crate::MetricsCell;
use prometheus::{IntCounterVec, register_int_counter_vec};

static CACHE_STATS_METRICS: MetricsCell<CacheStatsMetrics> = MetricsCell::new();
crate::register_metrics!(CacheStatsMetrics, CACHE_STATS_METRICS);

#[derive(Debug)]
struct CacheStatsMetrics {
    total: IntCounterVec,
}

impl CacheStatsMetrics {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            total: register_int_counter_vec!(
                "stonfi_metrics_cache_stats_total",
                "Cache request and miss counters",
                &["cache_name", "result"]
            )?,
        })
    }
}

/// Global cache request and miss counter.
///
/// The metric is registered by `init_metrics!` / `init_metrics_impl` as
/// `stonfi_metrics_cache_stats_total` with `cache_name` and `result` labels.
pub struct CacheStatsMetric;

impl CacheStatsMetric {
    /// Increment the request counter for `cache_name`.
    pub fn inc_request(cache_name: &str) {
        inc(cache_name, "request");
    }

    /// Increment the miss counter for `cache_name`.
    pub fn inc_miss(cache_name: &str) {
        inc(cache_name, "miss");
    }
}

fn inc(cache_name: &str, result: &str) {
    let Some(metrics) = CACHE_STATS_METRICS.get() else {
        return;
    };
    metrics.total.with_label_values(&[cache_name, result]).inc();
}
