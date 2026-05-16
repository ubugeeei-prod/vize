//! Main linter entry point.
//!
//! High-performance Vue template linter with arena allocation.
//! Split into:
//! - [`config`]: `Linter` struct, builder methods, and `LintResult`
//! - [`engine`]: Core linting methods and template extraction

mod config;
#[cfg(not(target_arch = "wasm32"))]
mod corsa_session;
mod engine;
#[cfg(not(target_arch = "wasm32"))]
mod native_type_aware;
pub(crate) mod script_rules;

pub use config::{LintResult, Linter};

#[cfg(test)]
mod tests;
