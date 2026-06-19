# stonfi_metrics

[![CI](https://github.com/ston-fi/stonfi-metrics/actions/workflows/build.yml/badge.svg)](https://github.com/ston-fi/stonfi-metrics/actions/workflows/build.yml)

`stonfi_metrics` provides small Prometheus helpers used by STON.fi services:

- `init_metrics!` starts an Axum `/metrics` endpoint and registers base build
  metadata labels;
- `init_metrics_impl` starts the same endpoint with explicit version, commit,
  and author labels;
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
