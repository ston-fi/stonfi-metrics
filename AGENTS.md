# AGENTS.md

## Purpose

This repository contains public Rust crates shared by STON.fi projects. Keep
changes small, publishable, and easy for downstream service agents to consume.

## Workspace Layout

- Root `Cargo.toml`: workspace manifest and shared dependency versions.
- `README.md`: short workspace overview and basic crate examples only.
- `crates/metrics`: `stonfi_metrics`, a Prometheus helper crate with an Axum
  `/metrics` server, base uptime metrics, cache counters, and duration tracking.

When working inside a crate, read that crate's `AGENTS.md` and `README.md`
before changing code or public docs.

## Public Crate Expectations

- Preserve public API compatibility unless the task explicitly asks for a
  breaking change.
- Keep public APIs documented with short rustdoc explaining capability,
  lifecycle, and intended use.
- Keep crate-specific user docs in the crate README. The root README should stay
  generic and point to crate-level details.
- When logic, public API, crate capabilities, or workspace crate membership
  changes, review and update all related docs in the same change. This includes
  root and crate-level `AGENTS.md`, root and crate-level `README.md`, and
  rustdoc for affected public items.
- If crate manifest metadata, README paths, or package includes change, verify
  the packaged crate content before reporting completion.
- Do not commit local build artifacts, `target/`, editor files, or `Cargo.lock`
  unless the repository policy changes.

## Validation

For docs-only changes, run at least:

```bash
git diff --check
cargo +nightly fmt --check
```

For crate code, public API, manifest, README, or packaging changes, run:

```bash
cargo test -p stonfi_metrics
cargo check -p stonfi_metrics --example simple_init
cargo +nightly fmt --check
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p stonfi_metrics --no-deps
cargo package -p stonfi_metrics --allow-dirty --list
cargo publish -p stonfi_metrics --dry-run --allow-dirty
```

Use the smallest meaningful subset only when the change cannot affect code,
docs rendering, or package contents, and state that reduced scope explicitly.
