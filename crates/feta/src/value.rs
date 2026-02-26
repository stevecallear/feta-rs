use std::fmt;

use serde::{Deserialize, Serialize};

/// The type of a feature value, which can be an integer, float, boolean, or string.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    #[serde(alias = "int")]
    Integer,
    #[serde(alias = "float")]
    Float,
    #[serde(alias = "bool")]
    Boolean,
    #[serde(alias = "string")]
    String,
}

impl fmt::Display for ValueType {
    /// Formats the `ValueType` as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer => f.write_str("integer"),
            Self::Float => f.write_str("float"),
            Self::Boolean => f.write_str("boolean"),
            Self::String => f.write_str("string"),
        }
    }
}

/// The value of a feature variant, which can be null, an integer, a float, a boolean, or a string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl Value {
    /// Checks if the `Value` has the specified `ValueType`.
    pub(crate) fn has_type(&self, t: &ValueType) -> bool {
        matches!(
            (self, t),
            (Value::Integer(_), ValueType::Integer)
                | (Value::Float(_), ValueType::Float)
                | (Value::Boolean(_), ValueType::Boolean)
                | (Value::String(_), ValueType::String)
        )
    }
}

impl From<i64> for Value {
    /// Converts an `i64` into a `Value::Integer`.
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<f64> for Value {
    /// Converts an `f64` into a `Value::Float`.
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<bool> for Value {
    /// Converts a `bool` into a `Value::Boolean`.
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<&str> for Value {
    /// Converts a string slice into a `Value::String`.
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<String> for Value {
    /// Converts a `String` into a `Value::String`.
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_deserialize() {
        let input = r#"["int", "integer", "float", "bool", "boolean", "string"]"#;
        let actual: Vec<ValueType> = serde_json::from_str(input).expect("should deserialize");
        let expected = vec![
            ValueType::Integer,
            ValueType::Integer,
            ValueType::Float,
            ValueType::Boolean,
            ValueType::Boolean,
            ValueType::String,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_value_deserialize() {
        let input = r#"[1, 1.1, true, false, "abc"]"#;
        let actual: Vec<Value> = serde_json::from_str(input).expect("should deserialize");
        let expected = vec![
            Value::Integer(1),
            Value::Float(1.1),
            Value::Boolean(true),
            Value::Boolean(false),
            Value::String("abc".to_string()),
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_value_serialize() {
        let input = vec![
            Value::Integer(1),
            Value::Float(1.1),
            Value::Boolean(true),
            Value::Boolean(false),
            Value::String("abc".to_string()),
        ];
        let actual = serde_json::to_string(&input).expect("should serialize");
        let expected = r#"[1,1.1,true,false,"abc"]"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_value_type_display() {
        let tests = vec![
            (ValueType::Integer, "integer"),
            (ValueType::Float, "float"),
            (ValueType::Boolean, "boolean"),
            (ValueType::String, "string"),
        ];

        for (input, expected) in tests {
            let actual = format!("{}", input);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_value_has_type() {
        let tests = vec![
            (Value::Integer(1), ValueType::Integer, true),
            (Value::Integer(1), ValueType::Float, false),
            (Value::Float(1.1), ValueType::Float, true),
            (Value::Float(1.1), ValueType::Boolean, false),
            (Value::Boolean(true), ValueType::Boolean, true),
            (Value::Boolean(true), ValueType::String, false),
            (Value::String(String::new()), ValueType::String, true),
            (Value::String(String::new()), ValueType::Integer, false),
        ];

        for (input, value_type, expected) in tests {
            let actual = input.has_type(&value_type);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_value_from_i64() {
        let actual = Value::from(1);
        assert_eq!(actual, Value::Integer(1));
    }

    #[test]
    fn test_value_from_f64() {
        let actual = Value::from(1.5);
        assert_eq!(actual, Value::Float(1.5));
    }

    #[test]
    fn test_value_from_bool() {
        let actual = Value::from(true);
        assert_eq!(actual, Value::Boolean(true));
    }

    #[test]
    fn test_value_from_str() {
        let actual = Value::from("abc");
        assert_eq!(actual, Value::String("abc".to_string()));
    }

    #[test]
    fn test_value_from_string() {
        let actual = Value::from("abc".to_string());
        assert_eq!(actual, Value::String("abc".to_string()));
    }
}
