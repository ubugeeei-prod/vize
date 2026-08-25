//! Completion provider for Vue SFC files.
//!
//! Provides context-aware completions for:
//! - Template expressions and directives
//! - Script bindings and imports
//! - CSS properties and Vue-specific selectors
//! - Real completions from Corsa (when available)
//!
//! Uses vize_croquis for accurate scope analysis and type information.
#![allow(clippy::disallowed_methods)]

mod dispatch;
mod items;
mod script;
mod service;
#[cfg(feature = "native")]
mod service_corsa_template;
mod service_inline_art;
#[cfg(feature = "native")]
mod static_object;
mod style;
pub(crate) mod template;

#[cfg(test)]
mod component_alias_props_tests;
#[cfg(test)]
mod component_lower_camel_props_tests;
#[cfg(test)]
mod component_props_tests;
#[cfg(test)]
mod dedup_tests;
#[cfg(test)]
mod split_component_props_tests;
#[cfg(test)]
mod static_object_tests;
#[cfg(test)]
mod template_event_tests;
// `insta`'s snapshot macros expand through the disallowed `std::format!`; the
// expansion is inside `insta`, so only an allow at the test module can silence
// it. See CONTRIBUTING.md, "Snapshot assertions in test targets".
#[allow(clippy::disallowed_macros)]
#[cfg(test)]
mod tests;

// Cross-module reuse: inlay-hint code resolves reactive binding types with
// the same heuristic that script completion uses.
pub(crate) use script::infer_reactive_value_type;

pub use dispatch::{CompletionService, TRIGGER_CHARACTERS, trigger_characters};
// Shared cursor-context predicates used by the block-specific handlers.
pub(crate) use dispatch::{
    is_inside_art_tag, is_inside_html_comment, is_inside_variant_tag, should_suggest_art_block,
    should_suggest_variant_block,
};
