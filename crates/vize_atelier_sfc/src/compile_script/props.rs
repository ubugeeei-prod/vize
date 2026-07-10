//! Props and emit type extraction utilities.
//!
//! This module handles extracting prop types from TypeScript type definitions
//! and processing withDefaults defaults.

mod ast_resolve;
mod defaults;
mod emits;
mod runtime_type;
mod text_resolve;
mod types;
mod validation;

pub use defaults::extract_with_defaults_defaults;
pub use emits::extract_emit_names_from_type;
pub use runtime_type::{
    add_null_to_runtime_type, is_valid_identifier, resolve_prop_js_type, strip_readonly_prefix,
};
pub use text_resolve::extract_prop_types_from_type;
pub use types::PropTypeInfo;
pub use validation::{validate_script_setup_semantics, validate_script_setup_semantics_located};

pub(crate) use defaults::normalize_destructure_default_value;
pub(crate) use runtime_type::{runtime_prop_key, ts_type_to_js_type};
pub(crate) use text_resolve::extract_prop_types_from_type_with_context;
pub(crate) use validation::validate_props_destructure_default_types;

#[cfg(test)]
mod tests;
