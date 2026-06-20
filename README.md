# stonfi_metrics

[![CI](https://github.com/ston-fi/stonfi-metrics/actions/workflows/build.yml/badge.svg)](https://github.com/ston-fi/stonfi-metrics/actions/workflows/build.yml)

`stonfi_metrics` provides small Prometheus helpers used by STON.fi services:

- `init_metrics!` starts an Axum `/metrics` endpoint and registers base build
  metadata labels;
- `init_metrics_impl` starts the same endpoint with explicit version, commit,
  and author labels;
- `register!(Metrics, METRICS)` registers module-owned metrics for automatic
  initialization during `init_metrics!` / `init_metrics_impl`;
- `MetricsServer::stop()` provides awaited shutdown, with best-effort shutdown
  on drop;
- `CacheStatsMetric` records cache request and miss counters;
- duration tracking helpers observe elapsed time into Prometheus histograms.

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

Use `register!(Metrics, METRICS)` when a crate or module owns private metrics
that should be initialized automatically on app startup.

```rust
use std::sync::OnceLock;

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
        Metrics::get()
            .requests_total
            .with_label_values(&[kind])
            .inc();
    }
}

static METRICS: OnceLock<Metrics> = OnceLock::new();

stonfi_metrics::register!(Metrics, METRICS);
```

The macro assumes `Metrics::new() -> anyhow::Result<Metrics>` and
`static METRICS: OnceLock<Metrics>`. It also generates a module-local
`Metrics::get() -> &'static Metrics` accessor for code that runs after metrics
startup. The struct, static, and helper methods can remain private to the
module.

## Duration Tracking

Use `track_duration!` with a `prometheus::Histogram` or `HistogramVec`.

```rust
let _timer = stonfi_metrics::track_duration!(request_duration, &["GET"]);
```

The timer observes elapsed milliseconds when dropped.

## Development

```bash
cargo test
cargo check --example simple_init
cargo +nightly fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```
