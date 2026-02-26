use std::collections::HashMap;

use feta::{Context, Reason, Value};
use serde::Deserialize;

pub const CONFIG: &str = include_str!("../tests/config.json");
const CASES_JSON: &str = include_str!("../tests/cases.json");

#[derive(Debug, Deserialize)]
struct TestCases {
    pub decide: Vec<DecideTestCase>,
    pub decide_all: Vec<DecideAllTestCase>,
}

#[derive(Debug, Deserialize)]
pub struct DecideTestCase {
    pub feature_key: String,
    pub context: Context,
    pub expected: Decision,
}

#[derive(Debug, Deserialize)]
pub struct DecideAllTestCase {
    pub context: Context,
    pub expected: HashMap<String, Decision>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Decision {
    pub variant: String,
    pub reason: Reason,
    pub value: Value,
    pub audience: Option<String>,
    pub has_error: bool,
}

pub fn decide_cases() -> Vec<DecideTestCase> {
    let cases: TestCases = serde_json::from_str(CASES_JSON).expect("failed to parse json");
    cases.decide
}

pub fn decide_all_cases() -> Vec<DecideAllTestCase> {
    let cases: TestCases = serde_json::from_str(CASES_JSON).expect("failed to parse json");
    cases.decide_all
}

pub fn assert_decision_eq(actual: &feta::Decision, expected: &Decision) {
    assert_eq!(actual.variant, expected.variant);
    assert_eq!(actual.reason, expected.reason);
    assert_eq!(actual.value, expected.value);
    assert_eq!(actual.audience, expected.audience);
    assert_eq!(actual.error.is_some(), expected.has_error);
}
