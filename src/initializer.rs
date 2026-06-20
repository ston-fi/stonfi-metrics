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

pub(crate) fn init_registered_metrics() -> anyhow::Result<()> {
    for initializer in inventory::iter::<MetricInitializer> {
        (initializer.init)().map_err(|error| {
            anyhow::anyhow!("failed to initialize {} metrics: {error}", initializer.name)
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    static TEST_REGISTER_MACRO_METRICS: crate::MetricsCell<TestRegisterMacroMetrics> =
        crate::MetricsCell::new();

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