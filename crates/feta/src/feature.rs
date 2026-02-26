use std::collections::HashMap;

use mexl::Environment;

use crate::{
    config,
    context::Context,
    decision::{Decision, DecisionBuilder},
    error::FetaError,
    hash,
    rule::Rule,
    value::{Value, ValueType},
    RuleBuilder,
};

/// The builder for constructing a `Feature` instance.
pub struct FeatureBuilder {
    name: Option<String>,
    enabled: bool,
    value_type: ValueType,
    variants: HashMap<String, Value>,
    default_variant: Option<String>,
    rules: Vec<Rule>,
    default_rule: Option<Rule>,
}

impl FeatureBuilder {
    /// Creates a new `FeatureBuilder` with the specified value type.
    pub fn new(value_type: ValueType) -> Self {
        Self {
            name: None,
            enabled: false,
            value_type,
            variants: HashMap::new(),
            default_variant: None,
            rules: Vec::new(),
            default_rule: None,
        }
    }

    /// Sets the name of the feature.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets whether the feature is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Adds a variant with the specified key and value to the feature.
    pub fn variant(mut self, key: impl Into<String>, value: Value) -> Self {
        self.variants.insert(key.into(), value);
        self
    }

    /// Sets the default variant for the feature.
    pub fn default_variant(mut self, key: impl Into<String>) -> Self {
        self.default_variant = Some(key.into());
        self
    }

    /// Sets the default rule for the feature.
    pub fn default_rule(mut self, rule: Rule) -> Self {
        self.default_rule = Some(rule);
        self
    }

    /// Adds an audience rule to the feature.
    pub fn audience_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Builds the `Feature` instance with the current values.
    pub fn build(mut self) -> Result<Feature, FetaError> {
        for value in self.variants.values() {
            if !value.has_type(&self.value_type) {
                return Err(FetaError::Configuration(format!(
                    "all variants must have type: {}",
                    self.value_type
                )));
            }
        }

        let default_variant = self.default_variant.ok_or(FetaError::Configuration(
            "default variant is required".to_string(),
        ))?;

        let default_value = match self.variants.get(&default_variant) {
            Some(v) => v.clone(),
            None => {
                return Err(FetaError::Configuration(format!(
                    "default variant does not exist: {}",
                    default_variant
                )));
            }
        };

        let default_rule = self.default_rule.ok_or(FetaError::Configuration(
            "default rule is required".to_string(),
        ))?;
        if default_rule.program.is_some() {
            return Err(FetaError::Configuration(
                "default rule must not have an expression".to_string(),
            ));
        }

        self.rules.push(default_rule);

        for rule in self.rules.iter() {
            for variant in rule.referenced_variants() {
                if !self.variants.contains_key(variant) {
                    return Err(FetaError::Configuration(format!(
                        "rule uses undefined variant: {}",
                        variant
                    )));
                }
            }
        }

        Ok(Feature {
            name: self.name.ok_or(FetaError::Configuration(
                "feature name is required".to_string(),
            ))?,
            enabled: self.enabled,
            variants: self.variants,
            default_variant,
            default_value,
            rules: self.rules,
        })
    }
}

/// Creates a default rule from the given configuration.
fn default_rule_from_config(bucketing: &config::Bucketing) -> Result<Rule, FetaError> {
    new_rule_builder(bucketing).build()
}

/// Creates an audience rule from the given configuration.
fn audience_rule_from_config(
    audience: &str,
    bucketing: &config::Bucketing,
    expr: &str,
) -> Result<Rule, FetaError> {
    new_rule_builder(bucketing).audience(audience, expr).build()
}

/// Creates a `RuleBuilder` from the given bucketing configuration.
fn new_rule_builder(bucketing: &config::Bucketing) -> RuleBuilder {
    let mut builder = RuleBuilder::new();
    match bucketing {
        config::Bucketing::Variant { variant } => builder = builder.variant(variant.clone(), 100),
        config::Bucketing::Distribution { distribution } => {
            for (variant, percentage) in distribution {
                builder = builder.variant(variant, *percentage)
            }
        }
    };
    builder
}

/// The `Feature` struct represents a feature with its configuration and rules for evaluation.
pub struct Feature {
    name: String,
    enabled: bool,
    variants: HashMap<String, Value>,
    default_variant: String,
    default_value: Value,
    rules: Vec<Rule>,
}

impl Feature {
    /// Creates a `Feature` instance from the given name and configuration.
    pub fn from_config(name: &str, cfg: &config::Feature) -> Result<Self, FetaError> {
        let mut builder = FeatureBuilder::new(cfg.value_type)
            .name(name)
            .enabled(cfg.enabled)
            .default_variant(cfg.default_variant.clone())
            .default_rule(default_rule_from_config(&cfg.default_rule.bucketing)?);

        for (variant, value) in &cfg.variants {
            builder = builder.variant(variant, value.clone());
        }

        for rule in &cfg.audience_rules {
            builder = builder.audience_rule(audience_rule_from_config(
                &rule.name,
                &rule.bucketing,
                &rule.expression,
            )?)
        }

        builder.build()
    }

    /// Evaluates the feature for the given context and returns a `Decision` with the result.
    pub fn decide(&self, ctx: &Context) -> Decision {
        let mut builder = DecisionBuilder::new()
            .variant(&self.default_variant)
            .value(self.default_value.clone());

        let hash = hash::calculate(&self.name, &ctx.user_key);
        builder = builder.hash(hash);

        if !self.enabled {
            return builder.disabled();
        }

        let mut env = Environment::default();
        if let Some(attributes) = &ctx.attributes {
            for (key, value) in attributes {
                env.set(key, value.clone());
            }
        }

        for rule in &self.rules {
            let applicable = match rule.is_applicable(&env) {
                Ok(b) => b,
                Err(e) => return builder.error(e),
            };

            if applicable {
                let variant = &rule.get_variant(hash);
                if let Some(audience) = &rule.audience {
                    builder = builder.audience(audience);
                }

                match self.variant_value(variant) {
                    Ok(v) => return builder.variant(variant).value(v).success(rule.reason),
                    Err(e) => return builder.error(e),
                }
            }
        }

        builder.error(FetaError::Configuration(
            "no applicable rules defined".to_string(),
        ))
    }

    /// Retrieves the value for the specified variant, returning an error if the variant is not defined.
    fn variant_value(&self, variant: &str) -> Result<Value, FetaError> {
        match self.variants.get(variant) {
            Some(value) => Ok(value.clone()),
            None => Err(FetaError::Configuration(format!(
                "variant not defined: {}",
                variant
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{decision::Reason, RuleBuilder};

    use super::*;

    #[test]
    fn test_feature_builder() {
        let rule = RuleBuilder::new()
            .variant("a", 50)
            .variant("b", 50)
            .build()
            .expect("rule should build");

        let _ = FeatureBuilder::new(ValueType::Integer)
            .name("feature")
            .enabled(true)
            .variant("a", 1.into())
            .variant("b", 2.into())
            .default_variant("a")
            .default_rule(rule)
            .build()
            .expect("feature should build");
    }

    #[test]
    fn test_feature_builder_errors() {
        let tests = vec![
            FeatureBuilder::new(ValueType::Integer) // no name
                .variant("a", 1.into())
                .enabled(true)
                .default_variant("a")
                .default_rule(
                    RuleBuilder::new()
                        .variant("a", 100)
                        .build()
                        .expect("rule should build"),
                ),
            FeatureBuilder::new(ValueType::Integer) // no default rule
                .name("f1")
                .enabled(true)
                .variant("a", 1.into())
                .default_variant("a")
                .audience_rule(
                    RuleBuilder::new()
                        .variant("a", 100)
                        .build()
                        .expect("rule should build"),
                ),
            FeatureBuilder::new(ValueType::Integer) // default rule with expression
                .name("f1")
                .enabled(true)
                .variant("a", 1.into())
                .default_variant("a")
                .default_rule(
                    RuleBuilder::new()
                        .variant("a", 100)
                        .audience("beta", "true")
                        .build()
                        .expect("rule should build"),
                ),
            FeatureBuilder::new(ValueType::Integer) // no default variant
                .name("f1")
                .enabled(true)
                .variant("a", 1.into())
                .default_rule(
                    RuleBuilder::new()
                        .variant("a", 100)
                        .build()
                        .expect("rule should build"),
                ),
            FeatureBuilder::new(ValueType::Integer) // invalid default variant
                .name("f1")
                .enabled(true)
                .variant("a", 1.into())
                .default_variant("invalid")
                .default_rule(
                    RuleBuilder::new()
                        .variant("a", 100)
                        .build()
                        .expect("rule should build"),
                ),
            FeatureBuilder::new(ValueType::Integer) // variant mismatch
                .name("f1")
                .enabled(true)
                .variant("a", 1.into())
                .default_variant("a")
                .default_rule(
                    RuleBuilder::new()
                        .variant("b", 100)
                        .build()
                        .expect("rule should build"),
                ),
            FeatureBuilder::new(ValueType::Integer) // variant type mismatch
                .name("f1")
                .enabled(true)
                .variant("a", 1.into())
                .variant("b", "abc".into())
                .default_variant("a")
                .default_rule(
                    RuleBuilder::new()
                        .variant("a", 50)
                        .variant("b", 50)
                        .build()
                        .expect("rule should build"),
                ),
        ];

        for test in tests {
            let result = test.build();
            assert!(result.is_err())
        }
    }

    #[test]
    fn test_feature_from_config() {
        let config = config::Feature {
            enabled: true,
            value_type: ValueType::Integer,
            variants: BTreeMap::from([("a".to_string(), 1.into()), ("b".to_string(), 2.into())]),
            default_variant: "a".to_string(),
            default_rule: config::DefaultRule {
                bucketing: config::Bucketing::Distribution {
                    distribution: BTreeMap::from([("a".to_string(), 50), ("b".to_string(), 50)]),
                },
            },
            audience_rules: vec![config::AudienceRule {
                name: "beta".to_string(),
                expression: "beta".to_string(),
                bucketing: config::Bucketing::Variant {
                    variant: "b".to_string(),
                },
            }],
        };

        let feature = Feature::from_config("exp", &config);
        assert!(!feature.is_err())
    }

    #[test]
    fn test_feature_evaluate() {
        struct TestCase {
            context: Context,
            expected: Decision,
        }

        // var=key: a=g, b=a, c=b
        let feature = FeatureBuilder::new(ValueType::Integer)
            .name("exp")
            .enabled(true)
            .variant("a", 1.into())
            .variant("b", 2.into())
            .variant("c", 3.into())
            .variant("d", 4.into())
            .default_variant("a")
            .default_rule(
                RuleBuilder::new()
                    .variant("a", 34)
                    .variant("b", 33)
                    .variant("c", 33)
                    .build()
                    .expect("rule should build"),
            )
            .audience_rule(
                RuleBuilder::new()
                    .variant("d", 100)
                    .audience("beta", "beta")
                    .build()
                    .expect("rule should build"),
            )
            .audience_rule(
                RuleBuilder::new()
                    .variant("a", 1)
                    .variant("d", 99)
                    .audience("internal", "internal")
                    .build()
                    .expect("rule should build"),
            )
            .build()
            .expect("feature should build");

        let tests = vec![
            TestCase {
                context: Context::new("g"),
                expected: DecisionBuilder::new()
                    .value(1.into())
                    .variant("a")
                    .success(Reason::Split),
            },
            TestCase {
                context: Context::new("a"),
                expected: DecisionBuilder::new()
                    .value(2.into())
                    .variant("b")
                    .success(Reason::Split),
            },
            TestCase {
                context: Context::new("b"),
                expected: DecisionBuilder::new()
                    .value(3.into())
                    .variant("c")
                    .success(Reason::Split),
            },
            TestCase {
                context: serde_json::from_str(r#"{"user_key":"d","attributes":{"beta": true}}"#)
                    .expect("should deserialize"),
                expected: DecisionBuilder::new()
                    .value(4.into())
                    .variant("d")
                    .audience("beta")
                    .success(Reason::Match),
            },
            TestCase {
                context: serde_json::from_str(
                    r#"{"user_key":"d","attributes":{"internal": true}}"#,
                )
                .expect("should deserialize"),
                expected: DecisionBuilder::new()
                    .value(4.into())
                    .variant("d")
                    .audience("internal")
                    .success(Reason::MatchSplit),
            },
        ];

        for test in tests {
            let actual = feature.decide(&test.context);
            let mut expected = test.expected.clone();
            expected.hash = actual.hash;
            assert_eq!(actual, expected)
        }
    }
}
