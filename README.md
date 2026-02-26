# feta-rs

[![Build Status](https://github.com/stevecallear/feta-rs/actions/workflows/test.yml/badge.svg)](https://github.com/stevecallear/feta-rs/actions/workflows/test.yml)

feta-rs is a feature flag engine for Rust, supporting advanced audience targeting, deterministic bucketing, and analytics-friendly decision objects. It is designed for both native and WebAssembly (WASM) environments.

## Overview
- **Feature Flagging**: Define features with multiple variants, audience rules, and distribution logic.
- **Rule Evaluation**: Use the `mexl` expression language for flexible, dynamic audience targeting.
- **Deterministic Bucketing**: Assign users to variants using `murmur3` hashing for consistent results.
- **Decision Objects**: All evaluations return detailed `Decision` structs, including hash, variant, reason, and error info.
- **Serde Support**: All configs and results are serializable/deserializable for easy integration.

## WASM Support
The `feta-wasi` crate exposes the core feature flag engine to WASI-compatible WebAssembly environments, allowing feature evaluation and event tracking from WASM hosts. The WASM module exports feature evaluation functions and imports a `track_event` callback for analytics.

## Crates
- [`crates/feta`](crates/feta/): Core feature flag logic ([README](crates/feta/README.md))
- [`crates/wasi`](crates/wasi/): WASI/WebAssembly bindings ([README](crates/wasi/README.md))
- [`crates/integration`](crates/integration/): Integration tests and config samples

## Getting Started
- See the individual crate READMEs for usage, configuration, and integration examples.
- Build and test the workspace with:
  ```sh
  cargo build
  cargo test
  ```