//! Croquis drawer for Vue SFC semantics.
//!
//! A [`Drawer`] walks script and template inputs, then returns the [`Croquis`] it drew.
//! Existing `Analyzer` names are kept as compatibility aliases in [`crate::analyzer`].

mod core;
mod helpers;
mod options;
mod script;
mod template;

#[cfg(test)]
mod tests;

pub use core::Drawer;
pub use helpers::{
    VForScopeAliases, extract_identifiers_oxc, extract_inline_callback_params, extract_slot_props,
    is_builtin_directive, is_component_tag, is_keyword, parse_v_for_expression,
    parse_v_for_scope_expression, strip_js_comments,
};
pub use options::DrawerOptions;
