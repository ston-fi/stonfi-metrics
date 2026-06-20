# AGENTS.md

## Purpose

This repository contains the public `stonfi_metrics` Rust crate used by STON.fi
projects. Keep changes small, publishable, and easy for downstream service
agents to consume.

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
- `register_metrics!(Metrics, METRICS)`: registers module-owned metrics for
  automatic initialization during `init_metrics!` / `init_metrics_impl`. The
  consumer module keeps `Metrics`, `METRICS`, and setters private; the macro
  expects `Metrics::new() -> anyhow::Result<Metrics>` and
  `static METRICS: stonfi_metrics::MetricsCell<Metrics>`.
- `MetricsCell`: fallibly initialized metrics storage. It serializes
  initialization, stores successful metrics once, and dereferences to `Metrics`
  after startup so local setters can use direct field access.
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

## Public Crate Expectations

- Preserve public API compatibility unless the task explicitly asks for a
  breaking change.
- Keep public APIs documented with short rustdoc explaining capability,
  lifecycle, and intended use.
- Keep user docs in the root README; it is the published crate README for
  crates.io.
- When logic, public API, crate capabilities, or package layout changes, review
  and update all related docs in the same change. This includes `AGENTS.md`,
  `README.md`, and rustdoc for affected public items.
- If crate manifest metadata, README paths, or package includes change, verify
  the packaged crate content before reporting completion.
- Do not commit local build artifacts, `target/`, editor files, or `Cargo.lock`
  unless the repository policy changes.

## Design Notes

- The crate uses the default global Prometheus registry. Tests or examples that
  register extra metrics should use unique metric names to avoid process-global
  registration conflicts.
- Use `MetricsCell<T>` for fallible global metric storage. Do not add new
  `Mutex<Option<T>>`, `OnceLock<Result<T, _>>`, or `LazyLock<T>` patterns for
  metric handles.
- Base metrics initialize explicitly because they need startup metadata labels.
  Cache stats and consumer module metrics should use `register_metrics!`.
  Keep all metric storage on `MetricsCell::init` so registration errors surface
  from `init_metrics_impl`.
- Registered module metrics should use one `MetricsCell<Metrics>` per module and
  one `register_metrics!(Metrics, METRICS)` invocation. Keep metric handles private and
  expose only the module-level setters needed by local callers.
- Consumers should start the server through `init_metrics!` or
  `init_metrics_impl`; `MetricsServer::start` is intentionally crate-private.
- Keep metric names stable once published.
- Keep `src/metrics_cell.rs` focused on storage/initialization, and
  `src/initializer.rs` focused on inventory registration.
- Prefer deleting duplicate initialization code over adding new wrappers.
- Keep docs direct and concise.
- Do not broaden dependencies or add abstractions unless the crate gains a real
  public capability.

## AI Development Notes

- Start by reading this file, `README.md`, `src/lib.rs`, and the module being
  changed.
- For public API changes, update `README.md`, this guide, rustdoc, and the
  example in the same commit.
- For metrics changes, validate both startup initialization and post-startup
  field access. Tests that touch the global Prometheus registry must use unique
  metric names.
- Keep generated or local planning files out of the published crate unless the
  crate intentionally ships them.
- Prefer one small commit in a task worktree. Do not touch the active checkout
  when it contains unrelated user changes.

## Validation

For docs-only changes, run at least:

```bash
git diff --check
cargo +nightly fmt --check
```

For crate code, public API, manifest, README, or packaging changes, run:

```bash
cargo test
cargo check --example simple_init
cargo +nightly fmt --check
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
cargo package --allow-dirty --list
cargo publish --dry-run --allow-dirty
```

Use the smallest meaningful subset only when the change cannot affect code,
docs rendering, or package contents, and state that reduced scope explicitly.
