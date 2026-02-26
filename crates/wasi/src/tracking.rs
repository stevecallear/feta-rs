use feta_core::{Reason, Value};

/// The tracking event generated from a feature evaluation, containing details about the feature, user, and decision.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub feature_key: String,
    pub user_key: String,
    pub variant: String,
    pub reason: Reason,
    pub value: Value,
    pub audience: Option<String>,
}

impl Event {
    /// Creates a new `Event` from the given feature key, user key, and decision.
    pub(super) fn new(
        feature_key: impl Into<String>,
        user_key: impl Into<String>,
        decision: &feta_core::Decision,
    ) -> Self {
        Self {
            feature_key: feature_key.into(),
            user_key: user_key.into(),
            variant: decision.variant.clone(),
            reason: decision.reason,
            value: decision.value.clone(),
            audience: decision.audience.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_from_decision() {
        let decision = feta_core::DecisionBuilder::new()
            .variant("variant")
            .value(1.into())
            .audience("audience")
            .success(feta_core::Reason::Match);

        let actual = Event::new("feature", "user", &decision);

        let expected = Event {
            feature_key: "feature".to_string(),
            user_key: "user".to_string(),
            variant: "variant".to_string(),
            reason: feta_core::Reason::Match,
            value: 1.into(),
            audience: Some("audience".to_string()),
        };

        assert_eq!(actual, expected);
    }
}
