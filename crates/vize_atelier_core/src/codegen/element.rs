//! Element generation functions.
//!
//! Handles code generation for all element types including native elements,
//! components, slots, and template elements, both in block and non-block contexts.

pub(crate) mod block;
mod directives;
pub(crate) mod helpers;
mod inline;
mod v_once;

#[allow(unused_imports)]
pub use block::generate_element_block;
#[allow(unused_imports)]
pub use directives::{
    generate_custom_directives_closing, generate_vmodel_closing, generate_vshow_closing,
};
pub(crate) use helpers::is_whitespace_or_comment;
#[allow(unused_imports)]
pub use helpers::{
    generate_root_node, get_custom_directives, has_custom_directives, has_renderable_props,
    has_v_once, has_vmodel_directive, has_vshow_directive,
};
pub use inline::generate_element;
#[allow(unused_imports)]
pub use v_once::{generate_v_once_child, generate_v_once_element, generate_v_once_props};
