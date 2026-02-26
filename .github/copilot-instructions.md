# Copilot Instructions for feta-rs

## Project Overview
- **feta-rs** is a Rust workspace for feature flag evaluation, with modular crates:
  - `feta`: Core logic for feature flagging, rule evaluation, and decision making
  - `integration`: Integration tests and config samples
  - `wasi`: WASI bindings for WebAssembly
  

## Architecture & Data Flow
- **Feature Evaluation**: 
  - User context (`Context`) is hashed and used to select feature variants
  - Audience rules are evaluated for each feature; errors are reported but do not halt evaluation
  - If no rule matches, the default variant is used
- **Config Structure**: 
  - Features are defined in config files (see `config.rs` and JSON samples in `integration/tests/`)
  - Each feature has variants, audience rules, and bucketing/distribution logic
- **Decision Objects**: 
  - Results of evaluation are returned as `Decision` structs, including hash, variant, reason, and error info

## Developer Workflows
- **Build**: Run `cargo build` from workspace root
- **Test**: Run `cargo test` from workspace root or per crate
- **Debug**: Use standard Rust debugging tools; WASI crate can be tested with Wasmtime
- **Config Testing**: Use JSON configs in `integration/tests/` for end-to-end scenarios

## Project-Specific Patterns
- **Rule Evaluation**: Rules are built with `RuleBuilder` and evaluated using the `mexl` expression language
- **Error Handling**: Errors are surfaced in `Decision` results, not as panics
- **Hashing**: Uses `murmur3` for deterministic bucketing
- **Serde**: All configs and results are serializable/deserializable via `serde`

## Integration Points
- **External Dependencies**: 
  - `mexl` (expression language, custom logic)
  - `murmur3` (hashing)
  - `serde` (serialization)
- **WASI**: WASI bindings in `wasi/` for WebAssembly integration

## Key Files & Directories
- `crates/feta/src/`: Core logic (feature, rule, config, decision, context)
- `crates/integration/tests/`: Config samples and integration tests
- `crates/wasi/`: WASI bindings and tests

## Example Patterns
- To test a config or decision, add a scenario to `integration/tests/cases.json`, updating `integration/tests/config.json` as needed.

---
If any section is unclear or missing, please specify which part needs more detail or examples.
