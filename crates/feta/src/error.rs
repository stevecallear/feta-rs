use serde::Serialize;
use thiserror::Error;

/// The error type for feta, representing various kinds of errors that can occur during feature evaluation.
#[derive(Error, Debug, Clone, PartialEq, Serialize)]
pub enum FetaError {
    /// An error that occurs when the configuration is invalid or cannot be loaded.
    #[error("Configuration error: {0}")]
    Configuration(String),
    /// An error that occurs when the request is invalid or cannot be processed.
    #[error("Request error: {0}")]
    Request(String),
    /// An error that occurs when there is an issue with audience evaluation.
    #[error("Targeting error: {0}")]
    Targeting(String),
}
