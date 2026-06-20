//! Prometheus helpers for STON.fi services.
//!
//! The crate starts an Axum `/metrics` endpoint, registers base uptime metrics,
//! initializes module-owned metrics registered through [`register!`], and exposes
//! small helpers for cache statistics and duration tracking.

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
use crate::initializer::init_registered_metrics;
use crate::server::MetricsServer;
mod base_metrics;
mod cache_stats_metric;
mod initializer;

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

/// Register a module-owned metrics struct for automatic startup initialization.
///
/// The target module must provide `static METRICS: OnceLock<Metrics>` and
/// `Metrics::new() -> anyhow::Result<Metrics>`.
#[macro_export]
macro_rules! register {
    ($metrics_ty:ty, $metrics_static:ident) => {
        const _: () = {
            static __STONFI_METRICS_INIT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

            fn __stonfi_metrics_init() -> $crate::__private::anyhow::Result<()> {
                let _guard = __STONFI_METRICS_INIT_LOCK.lock().map_err(|_| {
                    $crate::__private::anyhow::anyhow!(
                        "metrics initializer lock poisoned: {}",
                        concat!(module_path!(), "::", stringify!($metrics_ty))
                    )
                })?;

                if $metrics_static.get().is_some() {
                    return Ok(());
                }

                $metrics_static.set(<$metrics_ty>::new()?).map_err(|_| {
                    $crate::__private::anyhow::anyhow!(
                        "metrics already initialized: {}",
                        concat!(module_path!(), "::", stringify!($metrics_ty))
                    )
                })?;

                Ok(())
            }

            $crate::__private::inventory::submit! {
                $crate::__private::MetricInitializer {
                    name: concat!(module_path!(), "::", stringify!($metrics_ty)),
                    init: __stonfi_metrics_init,
                }
            }
        };

        impl $metrics_ty {
            fn get() -> &'static Self {
                match $metrics_static.get() {
                    Some(metrics) => metrics,
                    None => panic!(
                        "metrics used before initialization: {}",
                        concat!(module_path!(), "::", stringify!($metrics_ty))
                    ),
                }
            }
        }
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
    init_registered_metrics()?;
    MetricsServer::start(listen_address).await
}

#[doc(hidden)]
pub mod __private {
    pub use crate::initializer::MetricInitializer;
    pub use anyhow;
    pub use inventory;
}
