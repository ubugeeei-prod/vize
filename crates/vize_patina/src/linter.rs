//! Main linter entry point.
//!
//! High-performance Vue template linter with arena allocation.
//! Split into:
//! - [`config`]: `Linter` struct, builder methods, and `LintResult`
//! - [`engine`]: Core linting methods and template extraction

mod category_config;
mod category_rules;
mod compatibility;
mod config;
#[cfg(not(target_arch = "wasm32"))]
mod corsa_session;
pub(crate) mod css_rules;
mod engine;
mod musea_config;
pub(crate) mod musea_rules;
#[cfg(not(target_arch = "wasm32"))]
mod native_type_aware;
mod restricted_rules;
mod rule_selection;
pub(crate) mod script_rules;
mod severity;

pub use config::{LintResult, Linter};

#[cfg(test)]
mod tests;
