//! Element generation functions.
//!
//! Handles code generation for all element types including native elements,
//! components, slots, and template elements, both in block and non-block contexts.

pub(crate) mod block;
mod directives;
pub(crate) mod helpers;
mod inline;
mod v_once;
pub use directives::{
    generate_custom_directives_closing, generate_vmodel_closing, generate_vshow_closing,
};
pub(crate) use helpers::is_whitespace_or_comment;
pub use helpers::{
    generate_root_node, has_custom_directives, has_v_once, has_vmodel_directive,
    has_vshow_directive,
};
pub use inline::generate_element;
pub use v_once::generate_v_once_cached;
