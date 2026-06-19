//! Prometheus helpers for STON.fi services.
//!
//! The crate starts an Axum `/metrics` endpoint, registers base uptime metrics,
//! and exposes small helpers for cache statistics and duration tracking.

/// Duration bucket constants for Prometheus histograms.
pub mod constants;
/// RAII helpers and macros for observing elapsed durations.
pub mod duration_tracker;
/// Metrics HTTP server handle.
pub mod server;
/// Formatting helpers used by metric recorders.
pub mod utils;

use crate::base_metrics::init_base_metrics;
use crate::cache_stats_metric::init_cache_stats_metric;
use crate::server::MetricsServer;
mod base_metrics;
mod cache_stats_metric;

pub use crate::cache_stats_metric::CacheStatsMetric;

/// Start a metrics server using compile-time package and CI metadata.
///
/// Use [`init_metrics_impl`] when version, commit, or author labels should be
/// provided explicitly by the caller.
#[macro_export]
macro_rules! init_metrics {
    ($listen_address:expr) => {
        $crate::init_metrics_impl(
            $listen_address,
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
            option_env!("CI_COMMIT_SHORT_SHA").unwrap_or("local"),
            option_env!("GITLAB_USER_EMAIL").unwrap_or("local-dev"),
        )
    };
}

/// Initialize global metrics and start the `/metrics` HTTP server.
///
/// This registers base uptime metrics and cache statistics once, then starts a
/// [`server::MetricsServer`] bound to `listen_address`.
pub async fn init_metrics_impl(
    listen_address: &str,
    version: &str,
    commit: &str,
    author: &str,
) -> anyhow::Result<MetricsServer> {
    init_base_metrics(version, commit, author)?;
    init_cache_stats_metric()?;
    MetricsServer::start(listen_address).await
}
