use std::collections::HashMap;

use mexl::Object;
use serde::{Deserialize, Serialize};

/// The context for a feature evaluation, including the user key and any additional attributes.
#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    pub user_key: String,
    pub attributes: Option<HashMap<String, Object>>,
}

impl Context {
    pub fn new(user_key: impl Into<String>) -> Self {
        Self {
            user_key: user_key.into(),
            attributes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        const KEY: &str = "key";
        let ctx = Context::new(KEY);
        assert_eq!(ctx.user_key, KEY);
        assert!(ctx.attributes.is_none());
    }
}
