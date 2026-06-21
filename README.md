# stonfi_metrics

[![CI](https://github.com/ston-fi/stonfi-metrics/actions/workflows/build.yml/badge.svg)](https://github.com/ston-fi/stonfi-metrics/actions/workflows/build.yml)

`stonfi_metrics` provides small Prometheus helpers used by STON.fi services:

- `init_metrics!` starts an Axum `/metrics` endpoint and registers base build
  metadata labels;
- `init_metrics_impl` starts the same endpoint with explicit version, commit,
  and author labels;
- `register_metrics!(Metrics, METRICS)` registers module-owned metrics for
  automatic initialization during `init_metrics!` / `init_metrics_impl`;
- `MetricsCell` stores fallibly initialized metrics and still allows direct
  field access after startup;
- `MetricsServer::stop()` provides awaited shutdown, with best-effort shutdown
  on drop;
- `CacheStatsMetric` records cache request and miss counters;
- duration tracking helpers observe elapsed time into Prometheus histograms and
  expose timing state when explicit guards are needed.

## Basic Usage

```rust
let metrics_server = stonfi_metrics::init_metrics!("127.0.0.1:9000").await?;

stonfi_metrics::CacheStatsMetric::inc_request("pool_cache");
stonfi_metrics::CacheStatsMetric::inc_miss("pool_cache");

metrics_server.stop().await?;
```

Use `init_metrics_impl` when the default compile-time metadata from
`init_metrics!` does not match the build environment.

## Metrics

`init_metrics!` and `init_metrics_impl` register
`stonfi_metrics_uptime_seconds` with `version`, `commit`, and `author` labels.

`CacheStatsMetric` records `stonfi_metrics_cache_stats_total` with:

- `cache_name`: cache identifier supplied by the caller;
- `result`: `request` or `miss`.

## Module-Owned Metrics

Use `register_metrics!(Metrics, METRICS)` when a crate or module owns private metrics
that should be initialized automatically on app startup.

```rust
use stonfi_metrics::MetricsCell;

pub(super) struct Metrics {
    requests_total: prometheus::IntCounterVec,
}

impl Metrics {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            requests_total: prometheus::register_int_counter_vec!(
                "my_module_requests_total",
                "Total module requests",
                &["kind"]
            )?,
        })
    }

    pub(super) fn inc_request(kind: &str) {
        METRICS.requests_total.with_label_values(&[kind]).inc();
    }
}

static METRICS: MetricsCell<Metrics> = MetricsCell::new();

stonfi_metrics::register_metrics!(Metrics, METRICS);
```

The macro assumes `Metrics::new() -> anyhow::Result<Metrics>` and
`static METRICS: MetricsCell<Metrics>`. `init_metrics!` / `init_metrics_impl`
initialize the cell before the metrics server starts, so code that runs after
startup can access fields directly through `METRICS`. Construction errors are
returned from startup instead of being hidden behind first metric use. The
struct, static, and helper methods can remain private to the module.

## Duration Tracking

Use `track_duration!` with a `prometheus::Histogram` or `HistogramVec`.

```rust
let _timer = stonfi_metrics::track_duration!(request_duration, &["GET"]);
```

The timer observes elapsed milliseconds when dropped. Use
`duration_tracker::DurationTracker` or `DurationTrackerVec` directly when code
needs to read `elapsed()`, inspect the original `start_time()`, or call
`DurationTrackerVec::update_labels()` before the guard is dropped.

## Development

```bash
cargo test
cargo check --example simple_init
cargo +nightly fmt --check
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
cargo package --allow-dirty --list
cargo publish --dry-run --allow-dirty
```
