pub use feta_core::{Reason, Value};

/// The decision made for a feature evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub hash: u32,
    pub variant: String,
    pub reason: Reason,
    pub value: Value,
    pub audience: Option<String>,
    pub error: Option<String>,
}

impl From<feta_core::Decision> for Decision {
    /// Converts a `feta::Decision` into a `Decision`.
    fn from(value: feta_core::Decision) -> Self {
        Decision {
            hash: value.hash,
            variant: value.variant,
            reason: value.reason,
            value: value.value,
            audience: value.audience,
            error: value.error.map(|e| e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use feta_core::FetaError;

    use super::*;

    #[test]
    fn test_decision_from() {
        let err = FetaError::Request("error".to_string());

        let input = feta_core::Decision {
            hash: 1,
            variant: "variant".to_string(),
            reason: Reason::Match,
            value: 2.into(),
            audience: Some("audience".to_string()),
            error: Some(err.clone()),
        };

        let expected = Decision {
            hash: 1,
            variant: "variant".to_string(),
            reason: Reason::Match,
            value: 2.into(),
            audience: Some("audience".to_string()),
            error: Some(err.to_string()),
        };

        let actual: Decision = input.into();
        assert_eq!(actual, expected);
    }
}
