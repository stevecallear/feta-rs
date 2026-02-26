use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::value::{Value, ValueType};

/// The configuration for all features.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub features: BTreeMap<String, Feature>,
}

/// The configuration for a single feature.
#[derive(Debug, Deserialize, Serialize)]
pub struct Feature {
    pub enabled: bool,
    pub value_type: ValueType,
    pub variants: BTreeMap<String, Value>,
    pub default_variant: String,
    #[serde(default)]
    pub audience_rules: Vec<AudienceRule>,
    pub default_rule: DefaultRule,
}

/// The configuration for the default feature rule, which applies when no audience rules match.
#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultRule {
    #[serde(flatten)]
    pub bucketing: Bucketing,
}

/// The configuration for an audience rule, which applies to a specific subset of users.
#[derive(Debug, Deserialize, Serialize)]
pub struct AudienceRule {
    pub name: String,
    pub expression: String,
    #[serde(flatten)]
    pub bucketing: Bucketing,
}

/// The configuration for how to bucket users into variants, either by specifying a single variant or by defining a distribution of variants.
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Bucketing {
    Variant { variant: String },
    Distribution { distribution: BTreeMap<String, u8> },
}
