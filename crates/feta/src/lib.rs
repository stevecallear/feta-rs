mod context;
mod decision;
mod error;
mod feature;
mod features;
mod rule;
mod value;

pub mod config;
pub mod hash;

pub use crate::context::Context;
pub use crate::decision::{Decision, DecisionBuilder, Reason};
pub use crate::error::FetaError;
pub use crate::feature::{Feature, FeatureBuilder};
pub use crate::features::Features;
pub use crate::rule::{Rule, RuleBuilder};
pub use crate::value::{Value, ValueType};

pub use mexl::Object;
