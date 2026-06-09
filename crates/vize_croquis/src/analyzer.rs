//! Compatibility re-exports for the croquis drawer.
//!
//! New code should prefer [`crate::drawer`] and [`crate::Drawer`]. This module
//! remains public so downstream imports such as `vize_croquis::analyzer::Analyzer`
//! keep working.

pub use crate::drawer::{
    Drawer as Analyzer, DrawerOptions as AnalyzerOptions, IdentifierRef, VForScopeAliases,
    extract_identifier_refs_oxc, extract_identifiers_oxc, extract_inline_callback_params,
    extract_slot_props, is_builtin_directive, is_component_tag, is_keyword, parse_v_for_expression,
    parse_v_for_scope_expression, strip_js_comments,
};
