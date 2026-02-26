use std::collections::HashMap;

use crate::{
    config,
    context::Context,
    decision::{Decision, DecisionBuilder},
    error::FetaError,
    hash, Feature,
};

/// The `Features` struct manages a collection of features.
#[derive(Default)]
pub struct Features {
    features: HashMap<String, Feature>,
}

impl Features {
    /// Creates a `Features` instance from the given configuration.
    pub fn from_config(cfg: &config::Config) -> Result<Self, FetaError> {
        let mut features = HashMap::with_capacity(cfg.features.len());

        for (name, cfg) in &cfg.features {
            features.insert(name.clone(), Feature::from_config(name, cfg)?);
        }

        Ok(Self { features })
    }

    /// Evaluates the specified feature for the given context and returns a `Decision` with the result.
    pub fn decide(&self, feature: &str, ctx: &Context) -> Decision {
        match self.features.get(feature) {
            Some(f) => f.decide(ctx),
            None => DecisionBuilder::new()
                .hash(hash::calculate(feature, &ctx.user_key))
                .error(FetaError::Request(format!("invalid feature: {}", feature))),
        }
    }

    /// Evaluates all features for the given context and returns a map of feature names to their corresponding `Decision` results.
    pub fn decide_all(&self, ctx: &Context) -> HashMap<String, Decision> {
        let mut results = HashMap::with_capacity(self.features.len());

        for (name, feature) in self.features.iter() {
            results.insert(name.clone(), feature.decide(ctx));
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{
        config,
        decision::Reason,
        value::{Value, ValueType},
    };

    #[test]
    fn test_features_evaluate_success() {
        let config = get_config();
        let features = Features::from_config(&config).unwrap();
        let ctx = Context::new("g");

        let actual = features.decide("f1", &ctx);
        let mut expected = DecisionBuilder::new()
            .variant("a")
            .value(1.into())
            .success(Reason::Split);

        expected.hash = actual.hash;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_features_evaluate_error() {
        let config = get_config();
        let features = Features::from_config(&config).unwrap();
        let ctx = Context::new("g");

        let actual = features.decide("invalid", &ctx);

        assert!(actual.hash != 0);
        assert_eq!(actual.reason, Reason::Error);
        assert_eq!(actual.value, Value::Null);
        assert!(actual.error.is_some());
    }

    #[test]
    fn test_features_evaluate_all() {
        let config = get_config();
        let features = Features::from_config(&config).unwrap();
        let ctx = Context::new("g");

        let actual = features.decide_all(&ctx);
        let mut expected = HashMap::from([(
            "f1".to_string(),
            DecisionBuilder::new()
                .variant("a")
                .value(1.into())
                .success(Reason::Split),
        )]);

        for (key, expected) in expected.iter_mut() {
            expected.hash = actual.get(key).unwrap().hash;
        }

        assert_eq!(actual, expected);
    }

    fn get_config() -> config::Config {
        config::Config {
            features: BTreeMap::from([(
                "f1".to_string(),
                config::Feature {
                    enabled: true,
                    value_type: ValueType::Integer,
                    variants: BTreeMap::from([
                        ("a".to_string(), 1.into()),
                        ("b".to_string(), 2.into()),
                    ]),
                    default_variant: "a".to_string(),
                    default_rule: config::DefaultRule {
                        bucketing: config::Bucketing::Distribution {
                            distribution: BTreeMap::from([
                                ("a".to_string(), 50),
                                ("b".to_string(), 50),
                            ]),
                        },
                    },
                    audience_rules: vec![config::AudienceRule {
                        name: "beta".to_string(),
                        expression: "beta".to_string(),
                        bucketing: config::Bucketing::Variant {
                            variant: "b".to_string(),
                        },
                    }],
                },
            )]),
        }
    }
}
