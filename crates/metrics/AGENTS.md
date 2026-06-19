# AGENTS.md

## Crate Purpose

`stonfi_metrics` is a small Prometheus helper crate for STON.fi Rust services.
It owns service metrics initialization, the `/metrics` HTTP endpoint, base
uptime metadata, cache stats counters, and duration tracking helpers.

## Public Capabilities

- `init_metrics!(listen_address)`: starts the metrics server and uses
  compile-time package/CI metadata for base labels.
- `init_metrics_impl(listen_address, version, commit, author)`: same startup
  path with explicit base label values. Prefer this when the default CI metadata
  does not match the consumer's build environment.
- `server::MetricsServer`: handle returned by initialization. Use
  `listen_address()` for the bound address and `stop().await` for awaited
  shutdown. Dropping the handle only signals shutdown.
- `CacheStatsMetric`: global cache counter helper. Call after metrics
  initialization.
- `track_duration!`: RAII timer macro for `prometheus::Histogram` and
  `prometheus::HistogramVec`.
- `constants`: reusable histogram bucket presets in milliseconds.

## Exported Metrics

Base metrics:

- `stonfi_metrics_uptime_seconds`
- labels: `version`, `commit`, `author`

Cache stats:

- `stonfi_metrics_cache_stats_total`
- labels: `cache_name`, `result`
- result values: `request`, `miss`

## Design Notes

- The crate uses the default global Prometheus registry. Tests or examples that
  register extra metrics should use unique metric names to avoid process-global
  registration conflicts.
- Consumers should start the server through `init_metrics!` or
  `init_metrics_impl`; `MetricsServer::start` is intentionally crate-private.
- Keep metric names stable once published.
- Keep docs direct and concise. The crate README is the published README for
  crates.io.
- Do not broaden dependencies or add abstractions unless the crate gains a real
  public capability.

## Validation

For any change in this crate, prefer:

```bash
cargo test -p stonfi_metrics
cargo check -p stonfi_metrics --example simple_init
cargo +nightly fmt --check
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p stonfi_metrics --no-deps
```

For manifest, README, include-list, or publishing changes, also run:

```bash
cargo package -p stonfi_metrics --allow-dirty --list
cargo publish -p stonfi_metrics --dry-run --allow-dirty
```
