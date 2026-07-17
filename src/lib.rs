//! Prometheus helpers for STON.fi services.
//!
//! The crate starts an Axum `/metrics` endpoint, registers base uptime metrics,
//! initializes module-owned metrics registered through [`register_metrics!`], and exposes
//! small helpers for cache statistics and duration tracking.

// re-export
pub use prometheus;
/// Duration bucket constants for Prometheus histograms.
pub mod constants;
/// RAII helpers and macros for observing elapsed durations.
pub mod duration_tracker;
/// Metrics HTTP server handle.
pub mod server;
/// Formatting helpers used by metric recorders.
pub mod utils;
use crate::base_metrics::init_base_metrics;
use crate::initializer::init_registered_metrics;
use crate::server::MetricsServer;
mod base_metrics;
mod cache_stats_metric;
mod initializer;
mod metrics_cell;

pub use crate::cache_stats_metric::CacheStatsMetric;
pub use crate::metrics_cell::MetricsCell;

/// Start a metrics server using compile-time package and CI metadata.
///
/// Call without arguments to initialize metrics without starting the HTTP
/// server, such as in unit tests.
///
/// Use [`init_metrics_impl`] when version, commit, or author labels should be
/// provided explicitly by the caller.
#[macro_export]
macro_rules! init_metrics {
    () => {
        $crate::init_metrics_without_server_impl(
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
            option_env!("CI_COMMIT_SHORT_SHA").unwrap_or("local"),
            option_env!("GITLAB_USER_EMAIL").unwrap_or("local-dev"),
        )
    };
    ($listen_address:expr) => {
        $crate::init_metrics_impl(
            $listen_address,
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
            option_env!("CI_COMMIT_SHORT_SHA").unwrap_or("local"),
            option_env!("GITLAB_USER_EMAIL").unwrap_or("local-dev"),
        )
    };
}

/// Register a module-owned metrics struct for automatic startup initialization.
///
/// The target module must provide `static METRICS: MetricsCell<Metrics>` and
/// `Metrics::new() -> anyhow::Result<Metrics>`.
#[macro_export]
macro_rules! register_metrics {
    ($metrics_ty:ty, $metrics_static:ident) => {
        const _: () = {
            fn __stonfi_metrics_init() -> $crate::__private::anyhow::Result<()> {
                $metrics_static.init(concat!(module_path!(), "::", stringify!($metrics_ty)), <$metrics_ty>::new)
            }

            $crate::__private::inventory::submit! {
                $crate::__private::MetricInitializer {
                    name: concat!(module_path!(), "::", stringify!($metrics_ty)),
                    init: __stonfi_metrics_init,
                }
            }
        };
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
    init_metrics_without_server_impl(version, commit, author)?;
    MetricsServer::start(listen_address).await
}

#[doc(hidden)]
pub fn init_metrics_without_server_impl(version: &str, commit: &str, author: &str) -> anyhow::Result<()> {
    init_base_metrics(version, commit, author)?;
    init_registered_metrics()
}

#[doc(hidden)]
pub mod __private {
    pub use crate::initializer::MetricInitializer;
    pub use anyhow;
    pub use inventory;
}
