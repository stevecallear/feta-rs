use std::sync::{OnceLock, RwLock};

use feta_core::Features;

mod tracking;
mod types;

/// The global registry for features.
static FEATURES: OnceLock<RwLock<Features>> = OnceLock::new();

/// Gets the global registry for features, initializing it if necessary.
fn get_registry() -> &'static RwLock<Features> {
    FEATURES.get_or_init(|| RwLock::new(Features::default()))
}

pub mod bindings {
    use feta_core::{config::Config, Context, DecisionBuilder, Features, FetaError};

    wit_bindgen::generate!({
        world: "feta-wasi",
        with: {
            "feta:wasi/types": crate::types,
            "feta:wasi/tracking/event": crate::tracking::Event,
        }
    });

    /// The WASI component that implements the `feta-wasi` world.
    pub struct Component;

    impl Guest for Component {
        /// Initializes the global registry with the given configuration JSON.
        fn init(config_json: String) -> Result<(), String> {
            let config: Config = serde_json::from_str(&config_json).map_err(|e| e.to_string())?;
            let features = Features::from_config(&config).map_err(|e| e.to_string())?;

            let registry = super::get_registry();
            let mut write_guard = registry.write().map_err(|e| e.to_string())?;

            *write_guard = features;

            Ok(())
        }

        /// Evaluates the specified feature for the given context JSON and returns a `Decision` with the result.
        fn decide(feature_key: String, ctx_json: String) -> Decision {
            let ctx: Context = match serde_json::from_str(&ctx_json) {
                Ok(e) => e,
                Err(e) => {
                    return DecisionBuilder::new()
                        .error(FetaError::Request(e.to_string()))
                        .into()
                }
            };

            let read_guard = match super::get_registry().read() {
                Ok(g) => g,
                Err(e) => {
                    return DecisionBuilder::new()
                        .error(FetaError::Request(e.to_string()))
                        .into()
                }
            };

            let decision = read_guard.decide(&feature_key, &ctx);

            #[cfg(not(test))]
            {
                use crate::{bindings::feta::wasi::tracking::track_event, tracking::Event};

                let event = Event::new(feature_key, ctx.user_key, &decision);
                track_event(&event);
            }

            decision.into()
        }

        /// Evaluates all features for the given context JSON and returns a list of feature names and their corresponding `Decision` results.
        fn decide_all(ctx_json: String) -> Result<Vec<(String, Decision)>, String> {
            let ctx: Context = serde_json::from_str(&ctx_json).map_err(|e| e.to_string())?;
            let read_guard = super::get_registry().read().map_err(|e| e.to_string())?;

            let decisions = read_guard.decide_all(&ctx);

            #[cfg(not(test))]
            {
                use crate::{bindings::feta::wasi::tracking::track_event, tracking::Event};

                for (feature_key, decision) in decisions.iter() {
                    let event = Event::new(feature_key, &ctx.user_key, decision);
                    track_event(&event);
                }
            }

            let result: Vec<(String, Decision)> = decisions
                .into_iter()
                .map(|(key, decision)| (key, decision.into()))
                .collect();

            Ok(result)
        }
    }

    export!(Component);
}

#[cfg(test)]
mod tests {
    use crate::bindings::{Component, Guest};
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_component() {
        let config = feta_integration::CONFIG.to_string();
        Component::init(config.clone()).expect("failed to initialize");

        // test multiple calls to init
        Component::init(config).expect("failed to re-initialize");

        // test error case should preserve previous state
        let is_err = Component::init("{".to_string()).is_err();
        assert!(is_err);

        // decide cases
        for test in feta_integration::decide_cases() {
            let context_json =
                serde_json::to_string(&test.context).expect("failed to serialize context");

            let actual = Component::decide(test.feature_key, context_json);
            assert_eq!(convert_decision(actual), test.expected)
        }

        // decide_all cases
        for test in feta_integration::decide_all_cases() {
            let context_json =
                serde_json::to_string(&test.context).expect("failed to serialize context");

            let decisions =
                Component::decide_all(context_json).expect("failed to invoke decide_all");

            let actual: HashMap<String, feta_integration::Decision> = decisions
                .into_iter()
                .map(|(k, v)| (k, convert_decision(v)))
                .collect();

            assert_eq!(actual, test.expected)
        }
    }

    fn convert_decision(decision: bindings::Decision) -> feta_integration::Decision {
        feta_integration::Decision {
            variant: decision.variant,
            reason: decision.reason,
            value: decision.value,
            audience: decision.audience,
            has_error: decision.error.is_some(),
        }
    }
}
