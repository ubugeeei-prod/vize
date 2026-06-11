//! Statement and variable processing for Vue scripts.
//!
//! Handles processing of:
//! - Variable declarations (const, let, var)
//! - Function and class declarations
//! - Import and export statements
//! - Type declarations
//!
//! This module is split into:
//! - `statements`: Top-level statement and declaration processing
//! - `options_api`: Options API component metadata collection
//! - `class_component`: Class component (vue-class-component) extraction
//! - `macros`: Variable declarator processing (macros, reactivity, inject)
//! - `bindings`: Binding pattern helpers and expression classification

mod bindings;
mod class_component;
mod macros;
mod options_api;
mod statements;

pub(in crate::script_parser) use options_api::collect_options_api_component_metadata;
pub use statements::process_statement;
