# stonfi-rust

Shared public Rust crates for STON.fi projects.

[![CI](https://github.com/ston-fi/stonfi-rust/actions/workflows/build.yml/badge.svg)](https://github.com/ston-fi/stonfi-rust/actions/workflows/build.yml)

## Crates

### `stonfi_metrics`

Prometheus helper crate with an Axum `/metrics` server, base uptime metrics,
duration trackers, and cache statistics counters.

```rust
let metrics_server = stonfi_metrics::init_metrics!("127.0.0.1:9000").await?;

stonfi_metrics::CacheStatsMetric::inc_request("pool_cache");
stonfi_metrics::CacheStatsMetric::inc_miss("pool_cache");

metrics_server.stop().await?;
```

Base metrics are registered by `init_metrics!` and include
`stonfi_metrics_uptime_seconds`. `CacheStatsMetric` records
`stonfi_metrics_cache_stats_total` with `cache_name` and `result` labels.

See [`crates/metrics/README.md`](crates/metrics/README.md) for crate-level
details.

## Development

```bash
cargo test --lib --all
cargo check -p stonfi_metrics --example simple_init
cargo +nightly fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```
