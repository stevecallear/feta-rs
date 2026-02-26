# feta

`feta` is the core crate for feature flag evaluation in the feta-rs workspace. It provides the main logic for defining, evaluating, and managing feature flags, including support for audience targeting, bucketing, and variant distribution.

## Purpose
- Enable dynamic feature flagging and experimentation in Rust applications
- Support complex audience rules and deterministic bucketing
- Return detailed evaluation results for analytics and debugging

## Key Features
- **Feature Definition**: Define features with multiple variants, default values, and audience rules
- **Rule Evaluation**: Use the `mexl` expression language for flexible audience targeting
- **Bucketing**: Deterministic user bucketing using `murmur3` hashing
- **Decision Objects**: Evaluation returns a `Decision` struct with hash, variant, reason, and error info
- **Serde Support**: All configs and results are serializable/deserializable

## Usage
- Integrate by constructing `Features` from a config and calling `decide` or `decide_all` with a user `Context`
- See `src/config.rs` for config structure and `integration/tests/` for example configs

## Example
```rust
use feta::{Features, Context};

let config = /* load config from file or other source */;
let features = Features::from_config(&config).unwrap();
let ctx = Context::new("user123");
let decision = features.decide("my_feature", &ctx);
println!("Variant: {} (reason: {:?})", decision.variant, decision.reason);
```

## See Also
- [integration/tests/](../integration/tests/) for config samples and test cases