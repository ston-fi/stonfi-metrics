//! This stays an integration test because its always-failing inventory entry
//! must not share the process-wide registry with tests that initialize all metrics.

use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Context as _;
use stonfi_metrics::MetricsCell;

static SUCCESSFUL_INIT_COUNT: AtomicUsize = AtomicUsize::new(0);
static SUCCESSFUL_METRICS: MetricsCell<SuccessfulMetrics> = MetricsCell::new();

struct SuccessfulMetrics {
    value: u8,
}

impl SuccessfulMetrics {
    fn new() -> anyhow::Result<Self> {
        SUCCESSFUL_INIT_COUNT.fetch_add(1, Ordering::Relaxed);
        Ok(Self { value: 7 })
    }
}

stonfi_metrics::register_metrics!(SuccessfulMetrics, SUCCESSFUL_METRICS);

static CONCURRENT_INIT_COUNT: AtomicUsize = AtomicUsize::new(0);
static CONCURRENT_METRICS: MetricsCell<ConcurrentMetrics> = MetricsCell::new();

struct ConcurrentMetrics {
    value: u8,
}

impl ConcurrentMetrics {
    fn new() -> anyhow::Result<Self> {
        CONCURRENT_INIT_COUNT.fetch_add(1, Ordering::Relaxed);
        Ok(Self { value: 9 })
    }
}

stonfi_metrics::register_metrics!(ConcurrentMetrics, CONCURRENT_METRICS);

static FAILING_INIT_COUNT: AtomicUsize = AtomicUsize::new(0);
static FAILING_METRICS: MetricsCell<FailingMetrics> = MetricsCell::new();

struct FailingMetrics;

impl FailingMetrics {
    fn new() -> anyhow::Result<Self> {
        FAILING_INIT_COUNT.fetch_add(1, Ordering::Relaxed);
        Err(anyhow::anyhow!("root collector failure")).context("nested constructor context")
    }
}

stonfi_metrics::register_metrics!(FailingMetrics, FAILING_METRICS);

static UNREGISTERED_METRICS: MetricsCell<UnregisteredMetrics> = MetricsCell::new();

struct UnregisteredMetrics;

#[test]
fn test_lazy_initialization() -> anyhow::Result<()> {
    assert!(SUCCESSFUL_METRICS.get().is_none());
    assert!(CONCURRENT_METRICS.get().is_none());
    assert!(FAILING_METRICS.get().is_none());

    assert_eq!(SUCCESSFUL_METRICS.value, 7);
    assert_eq!(SUCCESSFUL_METRICS.value, 7);
    assert_eq!(SUCCESSFUL_INIT_COUNT.load(Ordering::Relaxed), 1);
    assert!(CONCURRENT_METRICS.get().is_none());
    assert!(FAILING_METRICS.get().is_none());

    let threads = (0..8)
        .map(|_| std::thread::spawn(|| CONCURRENT_METRICS.value))
        .collect::<Vec<_>>();
    for thread in threads {
        let value = thread
            .join()
            .map_err(|_| anyhow::anyhow!("concurrent metrics access panicked"))?;
        assert_eq!(value, 9);
    }
    assert_eq!(CONCURRENT_INIT_COUNT.load(Ordering::Relaxed), 1);

    let first_failure = std::panic::catch_unwind(|| {
        let _ = &*FAILING_METRICS;
    });
    assert!(FAILING_METRICS.get().is_none());
    assert_eq!(FAILING_INIT_COUNT.load(Ordering::Relaxed), 1);
    let first_message = panic_message(first_failure);
    assert!(first_message.contains("nested constructor context"));
    assert!(first_message.contains("root collector failure"));

    let second_failure = std::panic::catch_unwind(|| {
        let _ = &*FAILING_METRICS;
    });
    assert!(FAILING_METRICS.get().is_none());
    assert_eq!(FAILING_INIT_COUNT.load(Ordering::Relaxed), 2);
    let second_message = panic_message(second_failure);
    assert!(second_message.contains("nested constructor context"));
    assert!(second_message.contains("root collector failure"));

    let unregistered_failure = std::panic::catch_unwind(|| {
        let _ = &*UNREGISTERED_METRICS;
    });
    assert!(panic_message(unregistered_failure).contains("is not registered"));

    Ok(())
}

fn panic_message(result: Result<(), Box<dyn Any + Send>>) -> String {
    match result {
        Ok(()) => String::new(),
        Err(payload) => match payload.downcast::<String>() {
            Ok(message) => *message,
            Err(payload) => match payload.downcast::<&'static str>() {
                Ok(message) => (*message).to_owned(),
                Err(_) => String::new(),
            },
        },
    }
}
