use crate::metrics_cell::MetricsCell;
use anyhow::Context as _;

/// Registered initializer for module-owned Prometheus metrics.
///
/// Initializers are submitted by [`crate::register_metrics!`] and executed by
/// [`crate::init_metrics_impl`] before the metrics HTTP server starts.
pub struct MetricInitializer {
    /// Human-readable initializer name used in startup errors.
    pub name: &'static str,
    /// Fallible metric registration function.
    pub init: fn() -> anyhow::Result<()>,
}

inventory::collect!(MetricInitializer);

/// Registered initializer used to lazily initialize one exact metrics cell.
#[doc(hidden)]
pub struct LazyMetricInitializer {
    /// Human-readable initializer name used in warnings and errors.
    pub name: &'static str,
    /// Returns whether this initializer owns the erased metrics cell.
    pub matches_cell: fn(*const ()) -> bool,
    /// Initializes the cell and returns whether this call performed initialization.
    pub init: fn() -> anyhow::Result<bool>,
}

inventory::collect!(LazyMetricInitializer);

pub(crate) fn init_registered_metrics() -> anyhow::Result<()> {
    for initializer in inventory::iter::<MetricInitializer> {
        (initializer.init)()
            .map_err(|error| anyhow::anyhow!("failed to initialize {} metrics: {error}", initializer.name))?;
    }

    Ok(())
}

pub(crate) fn init_registered_metric<T>(cell: &MetricsCell<T>) -> anyhow::Result<&T> {
    let cell_ptr = std::ptr::from_ref(cell).cast::<()>();
    let initializer = inventory::iter::<LazyMetricInitializer>
        .into_iter()
        .find(|initializer| (initializer.matches_cell)(cell_ptr))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "metrics cell {} is not registered with register_metrics!",
                std::any::type_name::<T>()
            )
        })?;

    let initialized =
        (initializer.init)().with_context(|| format!("failed to initialize {} metrics", initializer.name))?;

    if initialized {
        tracing::warn!(
            metrics = initializer.name,
            "metrics accessed before explicit initialization; initialized on first use"
        );
    }

    cell.get().ok_or_else(|| {
        anyhow::anyhow!(
            "{} metrics initializer completed without initializing its cell",
            initializer.name
        )
    })
}

#[cfg(test)]
mod tests {
    static TEST_REGISTER_MACRO_METRICS: crate::MetricsCell<TestRegisterMacroMetrics> = crate::MetricsCell::new();

    struct TestRegisterMacroMetrics {
        counter: prometheus::IntCounter,
    }

    impl TestRegisterMacroMetrics {
        fn new() -> anyhow::Result<Self> {
            Ok(Self {
                counter: prometheus::register_int_counter!(
                    "stonfi_metrics_register_macro_test_total",
                    "Test counter registered through stonfi_metrics::register_metrics"
                )?,
            })
        }
    }

    crate::register_metrics!(TestRegisterMacroMetrics, TEST_REGISTER_MACRO_METRICS);

    #[test]
    fn test_register_macro_initializes_metrics() -> anyhow::Result<()> {
        super::init_registered_metrics()?;

        TEST_REGISTER_MACRO_METRICS.counter.inc();

        assert_eq!(TEST_REGISTER_MACRO_METRICS.counter.get(), 1);
        Ok(())
    }

    #[test]
    fn test_registered_metrics_initialization_is_idempotent() -> anyhow::Result<()> {
        super::init_registered_metrics()?;
        super::init_registered_metrics()?;

        let _ = &TEST_REGISTER_MACRO_METRICS.counter;
        Ok(())
    }

    #[test]
    fn test_init_metrics_without_server_runs_registered_initializers() -> anyhow::Result<()> {
        crate::init_metrics!()?;

        let _ = &TEST_REGISTER_MACRO_METRICS.counter;
        Ok(())
    }

    #[tokio::test]
    async fn test_init_metrics_impl_runs_registered_initializers() -> anyhow::Result<()> {
        let server = crate::init_metrics_impl(
            "127.0.0.1:0",
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
            option_env!("CI_COMMIT_SHORT_SHA").unwrap_or("local"),
            option_env!("GITLAB_USER_EMAIL").unwrap_or("local-dev"),
        )
        .await?;

        let _ = &TEST_REGISTER_MACRO_METRICS.counter;

        server.stop().await?;
        Ok(())
    }
}
