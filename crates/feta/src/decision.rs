use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{error::FetaError, Value};

/// The reason for a feature decision.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Unknown,
    Disabled,
    Static,
    Split,
    Match,
    MatchSplit,
    Error,
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Unknown => "unknown",
            Self::Disabled => "disabled",
            Self::Static => "static",
            Self::Split => "split",
            Self::Match => "match",
            Self::MatchSplit => "match_split",
            Self::Error => "error",
        };
        f.write_str(str)
    }
}

/// The result of a feature evaluation, including the variant, reason, and any error information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Decision {
    pub hash: u32,
    pub variant: String,
    pub reason: Reason,
    pub value: Value,
    pub audience: Option<String>,
    pub error: Option<FetaError>,
}

/// A builder for constructing `Decision` instances.
pub struct DecisionBuilder {
    hash: u32,
    variant: Option<String>,
    reason: Reason,
    value: Value,
    audience: Option<String>,
    error: Option<FetaError>,
}

impl DecisionBuilder {
    /// Creates a new `DecisionBuilder` with default values.
    pub fn new() -> Self {
        Self {
            hash: 0,
            variant: None,
            reason: Reason::Unknown,
            value: Value::Null,
            audience: None,
            error: None,
        }
    }

    /// Sets the hash value for the decision.
    pub fn hash(mut self, hash: u32) -> Self {
        self.hash = hash;
        self
    }

    /// Sets the variant for the decision.
    pub fn variant(mut self, variant: &str) -> Self {
        self.variant = Some(variant.to_string());
        self
    }

    /// Sets the value for the decision.
    pub fn value(mut self, value: Value) -> Self {
        self.value = value;
        self
    }

    /// Sets the audience for the decision.
    pub fn audience(mut self, audience: &str) -> Self {
        self.audience = Some(audience.to_string());
        self
    }

    /// Builds the decision as disabled.
    pub fn disabled(mut self) -> Decision {
        self.reason = Reason::Disabled;
        self.build()
    }

    /// Builds the decision as successful, with the specified reason.
    pub fn success(mut self, reason: Reason) -> Decision {
        self.reason = reason;
        self.build()
    }

    /// Builds the decision as an error, with the specified error information.
    pub fn error(mut self, err: FetaError) -> Decision {
        self.reason = Reason::Error;
        self.error = Some(err);
        self.build()
    }

    /// Builds the `Decision` instance with the current values.
    fn build(self) -> Decision {
        Decision {
            hash: self.hash,
            variant: self.variant.unwrap_or_default(),
            reason: self.reason,
            value: self.value,
            audience: self.audience,
            error: self.error,
        }
    }
}

impl Default for DecisionBuilder {
    /// Creates a new `DecisionBuilder` with default values.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reason_display() {
        let tests = vec![
            (Reason::Unknown, "unknown"),
            (Reason::Disabled, "disabled"),
            (Reason::Static, "static"),
            (Reason::Split, "split"),
            (Reason::Match, "match"),
            (Reason::MatchSplit, "match_split"),
            (Reason::Error, "error"),
        ];

        for (input, expected) in tests {
            let actual = format!("{}", input);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_reason_serialize() {
        let input = vec![
            Reason::Unknown,
            Reason::Disabled,
            Reason::Static,
            Reason::Split,
            Reason::Match,
            Reason::MatchSplit,
            Reason::Error,
        ];
        let actual = serde_json::to_string(&input).expect("should serialize");
        let expected = r#"["unknown","disabled","static","split","match","match_split","error"]"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decision_builder_success() {
        let actual = DecisionBuilder::new()
            .hash(1)
            .variant("var")
            .value(true.into())
            .audience("aud")
            .success(Reason::Match);
        let expected = Decision {
            hash: 1,
            variant: "var".to_string(),
            reason: Reason::Match,
            value: true.into(),
            audience: Some("aud".to_string()),
            error: None,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decision_builder_disabled() {
        let actual = DecisionBuilder::new()
            .hash(1)
            .variant("var")
            .value(true.into())
            .disabled();
        let expected = Decision {
            hash: 1,
            variant: "var".to_string(),
            reason: Reason::Disabled,
            value: true.into(),
            audience: None,
            error: None,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decision_builder_error() {
        let err = FetaError::Request(String::new());
        let actual = DecisionBuilder::new()
            .hash(1)
            .variant("var")
            .value(true.into())
            .error(err.clone());
        let expected = Decision {
            hash: 1,
            variant: "var".to_string(),
            reason: Reason::Error,
            value: true.into(),
            audience: None,
            error: Some(err),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_decision_builder_default() {
        let actual = DecisionBuilder::default()
            .hash(1)
            .variant("var")
            .value(true.into())
            .audience("aud")
            .success(Reason::Match);

        let expected = Decision {
            hash: 1,
            variant: "var".to_string(),
            reason: Reason::Match,
            value: true.into(),
            audience: Some("aud".to_string()),
            error: None,
        };
        assert_eq!(actual, expected);
    }
}
