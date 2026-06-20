# Automatic Metrics Registration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `stonfi_metrics::register_metrics!(Metrics, METRICS)` so consumer modules can keep metric structs and statics private while `init_metrics_impl` initializes them automatically.

**Architecture:** `stonfi_metrics` uses `inventory` to collect module-local metric initializers linked into the final binary. The `register_metrics!` macro generates a private initializer, a private per-registration init lock, and a module-local `Metrics::get() -> &'static Metrics` accessor. `init_metrics_impl` initializes built-in metrics, runs all registered metric initializers, then starts the HTTP server.

**Tech Stack:** Rust 2024, `std::sync::OnceLock`, `inventory`, `anyhow`, Prometheus global registry, existing Axum metrics server.

---

## File Structure

- Modify `Cargo.toml`: add `inventory = "0.3"` and include nested `src/initializer/*` test sources in the package.
- Create `src/initializer.rs`: own `MetricInitializer`, `inventory::collect!`, `init_registered_metrics()`, and `#[cfg(test)] mod tests`.
- Create `src/initializer/tests.rs`: module-local unit tests for registration, idempotency, generated getter, and startup initialization.
- Modify `src/lib.rs`: define `register_metrics!`, call `init_registered_metrics()` from `init_metrics_impl`, and keep hidden macro support exports at the bottom of the file.
- Modify `examples/simple_init.rs`: replace `LazyLock<Option<Metrics>>` with `OnceLock<Metrics>`, register the metrics, and use generated `Metrics::get()`.
- Modify `README.md` and `AGENTS.md`: document the new API and private module-owned metrics pattern.

## Task 1: Initializer Infrastructure

**Files:**
- Modify: `Cargo.toml`
- Create: `src/initializer.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Add dependency and package include**

```toml
[dependencies]
anyhow = "1"
axum = "0.8"
inventory = "0.3"
parking_lot = "0.12"
prometheus = "0.14"
tokio = { version = "1", features = ["macros", "net", "rt", "rt-multi-thread", "sync", "time"] }
tracing = "0.1"
```

Add nested initializer tests to package includes:

```toml
include = [
    "AGENTS.md",
    "Cargo.toml",
    "README.md",
    "src/*",
    "src/initializer/*",
    "examples/*",
]
```

- [ ] **Step 2: Add initializer module**

```rust
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
mod tests;
```

- [ ] **Step 3: Wire initializer into crate root**

Use `mod initializer;`, import `initializer::init_registered_metrics`, and call it in `init_metrics_impl` after built-in base/cache metric initialization and before `MetricsServer::start`.

- [ ] **Step 4: Verify**

Run:

```bash
cargo check --lib
```

Expected: pass.

## Task 2: `register_metrics!` Macro

**Files:**
- Modify: `src/lib.rs`
- Create: `src/initializer/tests.rs`

- [ ] **Step 1: Add failing test for generated API**

Add a test metrics struct in `src/initializer/tests.rs`, invoke:

```rust
crate::register_metrics!(TestRegisterMacroMetrics, TEST_REGISTER_MACRO_METRICS);
```

Write tests that expect:

```rust
super::init_registered_metrics()?;
let metrics = TestRegisterMacroMetrics::get();
metrics.counter.inc();
assert_eq!(metrics.counter.get(), 1);
```

Run:

```bash
cargo test initializer::tests::test_register_macro_initializes_metrics -- --exact
```

Expected before implementation: compile failure because `register_metrics!` or generated `get()` is missing.

- [ ] **Step 2: Implement macro**

Add `register_metrics!` after `init_metrics!`:

```rust
#[macro_export]
macro_rules! register_metrics {
    ($metrics_ty:ty, $metrics_static:ident) => {
        const _: () = {
            static __STONFI_METRICS_INIT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

            fn __stonfi_metrics_init() -> $crate::__private::anyhow::Result<()> {
                let _guard = __STONFI_METRICS_INIT_LOCK.lock().map_err(|_| {
                    $crate::__private::anyhow::anyhow!(
                        "metrics initializer lock poisoned: {}",
                        concat!(module_path!(), "::", stringify!($metrics_ty))
                    )
                })?;

                if $metrics_static.get().is_some() {
                    return Ok(());
                }

                $metrics_static.set(<$metrics_ty>::new()?).map_err(|_| {
                    $crate::__private::anyhow::anyhow!(
                        "metrics already initialized: {}",
                        concat!(module_path!(), "::", stringify!($metrics_ty))
                    )
                })?;

                Ok(())
            }

            $crate::__private::inventory::submit! {
                $crate::__private::MetricInitializer {
                    name: concat!(module_path!(), "::", stringify!($metrics_ty)),
                    init: __stonfi_metrics_init,
                }
            }
        };

        impl $metrics_ty {
            fn get() -> &'static Self {
                match $metrics_static.get() {
                    Some(metrics) => metrics,
                    None => panic!(
                        "metrics used before initialization: {}",
                        concat!(module_path!(), "::", stringify!($metrics_ty))
                    ),
                }
            }
        }
    };
}
```

- [ ] **Step 3: Add hidden macro support exports at bottom of `src/lib.rs`**

```rust
#[doc(hidden)]
pub mod __private {
    pub use crate::initializer::MetricInitializer;
    pub use anyhow;
    pub use inventory;
}
```

- [ ] **Step 4: Verify focused tests**

Run:

```bash
cargo test initializer::tests -- --test-threads=1
```

Expected: initializer unit tests pass.

## Task 3: Example And Docs

**Files:**
- Modify: `examples/simple_init.rs`
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `src/lib.rs`

- [ ] **Step 1: Update example**

Use:

```rust
static METRICS: OnceLock<Metrics> = OnceLock::new();

stonfi_metrics::register_metrics!(Metrics, METRICS);
```

Use generated getter after startup:

```rust
Metrics::get()
    .requests_total
    .with_label_values(&[label])
    .inc();

let _timer = track_duration!(Metrics::get().request_duration, &["GET"]);
```

- [ ] **Step 2: Update docs**

Document that `register_metrics!(Metrics, METRICS)` expects `Metrics::new() -> anyhow::Result<Metrics>` and `static METRICS: OnceLock<Metrics>`, and generates module-local `Metrics::get() -> &'static Metrics` for code that runs after successful startup.

- [ ] **Step 3: Verify example and docs**

Run:

```bash
cargo check --example simple_init
cargo run --example simple_init
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
```

Expected: all pass; example prints the metrics endpoint and exits cleanly.

## Task 4: Full Validation And Commit

**Files:**
- All changed files.

- [ ] **Step 1: Run full validation**

```bash
cargo test
cargo check --example simple_init
cargo +nightly fmt --check
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
cargo package --allow-dirty --list
cargo publish --dry-run --allow-dirty
```

Expected: all commands pass. Package list includes `src/initializer.rs` and `src/initializer/tests.rs`; no ignored-test warning appears.

- [ ] **Step 2: Inspect diff**

```bash
git diff --check
git diff -- Cargo.toml src/lib.rs src/initializer.rs src/initializer/tests.rs examples/simple_init.rs README.md AGENTS.md
git status --short
```

Expected: no whitespace errors; only intended files changed.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml src/lib.rs src/initializer.rs src/initializer/tests.rs examples/simple_init.rs README.md AGENTS.md docs/superpowers/plans/2026-06-20-automatic-metrics-registration.md
git commit -m "metrics: add automatic metrics registration"
```

Expected: one validated commit.

## Self-Review

- Spec coverage: implements `stonfi_metrics::register_metrics!(Metrics, METRICS)`, automatic startup initialization, private metric handles, generated module-local getter, no proc-macro crate, and docs/examples.
- Placeholder scan: no unresolved placeholders or unspecified implementation steps remain.
- Type consistency: all snippets use `initializer`, `MetricInitializer`, `init_registered_metrics`, `register_metrics!`, `OnceLock<Metrics>`, and `Metrics::new() -> anyhow::Result<Self>` consistently.
