use parking_lot::Mutex;
use prometheus::{IntCounterVec, register_int_counter_vec};

static CACHE_STATS_METRICS: Mutex<Option<IntCounterVec>> = Mutex::new(None);

pub(super) fn init_cache_stats_metric() -> anyhow::Result<()> {
    let mut cache_stats = CACHE_STATS_METRICS.lock();
    if cache_stats.is_some() {
        return Ok(());
    }

    let metric = register_int_counter_vec!(
        "stonfi_metrics_cache_stats_total",
        "Cache request and miss counters",
        &["cache_name", "result"]
    )?;
    *cache_stats = Some(metric);
    Ok(())
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
    let cache_stats = CACHE_STATS_METRICS.lock();
    let Some(metric) = cache_stats.as_ref() else {
        return;
    };
    metric.with_label_values(&[cache_name, result]).inc();
}
