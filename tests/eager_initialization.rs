use std::sync::atomic::{AtomicUsize, Ordering};

use stonfi_metrics::MetricsCell;

static EAGER_INIT_COUNT: AtomicUsize = AtomicUsize::new(0);
static EAGER_METRICS: MetricsCell<EagerMetrics> = MetricsCell::new();

struct EagerMetrics {
    initialized: bool,
}

impl EagerMetrics {
    fn new() -> anyhow::Result<Self> {
        EAGER_INIT_COUNT.fetch_add(1, Ordering::Relaxed);
        Ok(Self { initialized: true })
    }
}

stonfi_metrics::register_metrics!(EagerMetrics, EAGER_METRICS);

#[test]
fn test_explicit_startup_initializes_registered_metrics() -> anyhow::Result<()> {
    assert!(EAGER_METRICS.get().is_none());

    stonfi_metrics::init_metrics!()?;

    let metrics = EAGER_METRICS
        .get()
        .ok_or_else(|| anyhow::anyhow!("explicit startup did not initialize registered metrics"))?;
    assert!(metrics.initialized);
    assert_eq!(EAGER_INIT_COUNT.load(Ordering::Relaxed), 1);

    Ok(())
}
