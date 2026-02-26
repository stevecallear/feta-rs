# feta-wasi

`feta-wasi` provides WASI-compatible WebAssembly bindings for the feta feature flag engine, enabling feature evaluation and event tracking in WASM environments.

## Purpose
- Expose feta's feature flag evaluation to WASM runtimes via the WASI component model
- Allow host applications to track feature evaluation events via an importable callback

## Exposed WASM Functions
The following functions are exported to the WASM host (see `src/lib.rs` and `wit/feta-wasi.wit`):

- `init(config_json: string) -> result<(), string>`: Initialize the feature registry with a JSON config
- `decide(feature_key: string, context_json: string) -> result<decision, string>`: Evaluate a single feature for a user context
- `decide_all(context_json: string) -> result<list<(string, decision)>, string>`: Evaluate all features for a user context

All JSON arguments must match the feta config and context schemas (see `feta_core::config::Config` and `feta_core::Context`).

## Event Tracking Import
The WASM module expects the host to provide a `track_event(event)` function (see `src/tracking.rs` and `wit/feta-wasi.wit`). This is called after each feature evaluation, allowing the host to capture analytics or audit events.

- `track_event(event: Event)`: Receives an event record with feature key, user key, variant, reason, value, and audience info.

## Example
See `tests/integration.rs` for a full example using Wasmtime, including event tracking.