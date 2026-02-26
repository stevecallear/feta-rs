use mexl::{Environment, Program};

use crate::{decision::Reason, error::FetaError};

/// The `RuleBuilder` struct provides a builder pattern for constructing `Rule` instances.
#[derive(Debug, Clone)]
pub struct RuleBuilder {
    percentages: Vec<(String, u8)>,
    audience: Option<(String, String)>,
    is_default: bool,
}

impl Default for RuleBuilder {
    /// Creates a new `RuleBuilder` instance default values.
    fn default() -> Self {
        RuleBuilder::new()
    }
}

impl RuleBuilder {
    /// Creates a new `RuleBuilder` instance with default values.
    pub fn new() -> Self {
        Self {
            percentages: Vec::new(),
            audience: None,
            is_default: false,
        }
    }

    /// Adds a variant with the specified percentage to the rule.
    pub fn variant(mut self, variant: impl Into<String>, percentage: u8) -> Self {
        self.percentages.push((variant.into(), percentage));
        self
    }

    /// Marks the rule as the default rule, which applies when no audience rules match.
    pub fn audience(mut self, audience: impl Into<String>, expression: impl Into<String>) -> Self {
        self.audience = Some((audience.into(), expression.into()));
        self
    }

    /// Builds the `Rule` instance from the provided configuration.
    pub fn build(self) -> Result<Rule, FetaError> {
        let mut bound: u32 = 0;
        let buckets: Vec<Bucket> = self
            .percentages
            .into_iter()
            .map(|(k, p)| {
                let b = Bucket {
                    variant: k.clone(),
                    lower_bound: bound,
                    upper_bound: bound + p as u32,
                };

                bound = b.upper_bound;
                b
            })
            .collect();

        if buckets.is_empty() || bound != 100 {
            return Err(FetaError::Configuration(
                "invalid variant configuration".to_string(),
            ));
        }

        let mut reason = match buckets.len() {
            0 => unreachable!(),
            1 => Reason::Static,
            _ => Reason::Split,
        };

        let mut program = None;
        let mut audience = None;
        if let Some((aud, expr)) = self.audience {
            if self.is_default {
                return Err(FetaError::Configuration(
                    "audience not permitted for default rule".to_string(),
                ));
            }

            audience = Some(aud);
            program = Some(mexl::compile(&expr).map_err(|e| FetaError::Targeting(e.to_string()))?);

            reason = match reason {
                Reason::Static => Reason::Match,
                Reason::Split => Reason::MatchSplit,
                _ => reason,
            };
        };

        Ok(Rule {
            buckets,
            program,
            reason,
            audience,
        })
    }
}

/// The `Rule` struct represents a targeting rule that determines how users are bucketed into variants based on their attributes and a hash value.
#[derive(Clone)]
pub struct Rule {
    buckets: Vec<Bucket>,
    pub(crate) program: Option<Program>,
    pub(crate) audience: Option<String>,
    pub(crate) reason: Reason,
}

/// Bucket configuration for a rule, defining the variant and the hash range that maps to that variant.
#[derive(Debug, Clone)]
pub struct Bucket {
    variant: String,
    lower_bound: u32,
    upper_bound: u32,
}

impl Rule {
    /// Evaluates whether the rule is applicable to the given environment by evaluating the audience expression if one exists or returning true if not.
    pub fn is_applicable(&self, env: &Environment) -> Result<bool, FetaError> {
        match &self.program {
            Some(p) => {
                let result = mexl::run(p, env).map_err(|e| FetaError::Targeting(e.to_string()))?;
                Ok(result == true.into())
            }
            None => Ok(true),
        }
    }

    /// Determines the variant for the given hash value based on the rule's bucket configuration.
    pub fn get_variant(&self, hash: u32) -> String {
        let hash_mod = hash % 100_u32;
        self.buckets
            .iter()
            .find(|b| hash_mod >= b.lower_bound && hash_mod < b.upper_bound)
            .expect("invalid bucket configuration") // this is unreachable if constructed via builder
            .variant
            .clone()
    }

    /// Returns an iterator over the variants that are referenced by this rule.
    pub(super) fn referenced_variants(&self) -> impl Iterator<Item = &String> + '_ {
        self.buckets.iter().map(|b| &b.variant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_builder_default_static() {
        let rule = RuleBuilder::default()
            .variant("a", 100)
            .build()
            .expect("rule should build");

        assert_eq!(rule.reason, Reason::Static);
    }

    #[test]
    fn test_rule_builder_default_split() {
        let rule = RuleBuilder::default()
            .variant("a", 50)
            .variant("b", 50)
            .build()
            .expect("rule should build");

        assert_eq!(rule.reason, Reason::Split)
    }

    #[test]
    fn test_rule_builder_audience() {
        let rule = RuleBuilder::default()
            .variant("a", 100)
            .audience("beta", "orders gt 10")
            .build()
            .expect("rule should build");

        assert_eq!(rule.reason, Reason::Match);
        assert_eq!(rule.audience.unwrap_or_default(), "beta");
    }

    #[test]
    fn test_rule_builder_audience_split() {
        let rule = RuleBuilder::default()
            .variant("a", 50)
            .variant("b", 50)
            .audience("beta", "orders gt 10")
            .build()
            .expect("rule should build");

        assert_eq!(rule.reason, Reason::MatchSplit)
    }

    #[test]
    fn test_rule_builder_errors() {
        let tests = vec![
            RuleBuilder::new() // rule with invalid percentages
                .variant("a", 50)
                .variant("b", 40),
            RuleBuilder::new() // rule with invalid expression
                .variant("a", 100)
                .audience("audience", "+2"), // mexl compile error
        ];

        for test in tests {
            let result = test.build();
            assert!(result.is_err())
        }
    }

    #[test]
    fn test_rule_is_applicable() {
        struct TestCase {
            builder: RuleBuilder,
            environment: Environment,
            expected: Result<bool, ()>,
        }

        let tests = vec![
            TestCase {
                // no expression evaluates to true
                builder: RuleBuilder::new().variant("a", 100),
                environment: Environment::default(),
                expected: Ok(true),
            },
            TestCase {
                // expression evaluates to true
                builder: RuleBuilder::new().variant("a", 100).audience("beta", "b"),
                environment: serde_json::from_str(r#"{"b": true}"#).unwrap(),
                expected: Ok(true),
            },
            TestCase {
                // expression evaluates to false
                builder: RuleBuilder::new().variant("a", 100).audience("beta", "b"),
                environment: serde_json::from_str(r#"{"b": false}"#).unwrap(),
                expected: Ok(false),
            },
            TestCase {
                // expression results in runtime error
                builder: RuleBuilder::new()
                    .variant("a", 100)
                    .audience("beta", "true.a"),
                environment: Environment::default(),
                expected: Err(()),
            },
        ];

        for test in tests {
            let rule = test.builder.build().expect("rule should build");
            let result = rule.is_applicable(&test.environment);
            assert_eq!(result.is_err(), test.expected.is_err());

            if let Ok(actual) = result {
                assert_eq!(actual, test.expected.unwrap());
            }
        }
    }

    #[test]
    fn test_rule_get_variant() {
        let rule = RuleBuilder::new()
            .variant("a", 50)
            .variant("b", 50)
            .build()
            .expect("rule should build");

        let tests = vec![(0, "a"), (49, "a"), (50, "b"), (51, "b"), (100, "a")];

        for (hash, expected) in tests {
            let actual = rule.get_variant(hash);
            assert_eq!(actual, expected.to_string());
        }
    }

    #[test]
    fn test_rule_referenced_variants() {
        let rule = RuleBuilder::new()
            .variant("a", 34)
            .variant("b", 33)
            .variant("c", 33)
            .build()
            .expect("rule should build");

        let expected = vec!["a", "b", "c"];
        let actual: Vec<&str> = rule.referenced_variants().map(|s| s.as_str()).collect();

        assert_eq!(actual, expected);
    }
}
