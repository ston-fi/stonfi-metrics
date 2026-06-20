/// Registered initializer for module-owned Prometheus metrics.
///
/// Initializers are submitted by [`crate::register!`] and executed by
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
mod tests;
