//! Helper functions for Vue template analysis.
//!
//! Provides utilities for:
//! - Component and directive detection (`keywords`)
//! - Identifier extraction from expressions (`identifiers`)
//! - v-for expression parsing (`v_for`)
//! - v-slot and inline callback parameter extraction (`slots`)

mod identifiers;
mod keywords;
mod slots;
mod v_for;

pub use identifiers::{extract_identifiers_oxc, strip_js_comments};
pub use keywords::{is_builtin_directive, is_component_tag, is_keyword};
pub use slots::{extract_inline_callback_params, extract_slot_props};
pub use v_for::parse_v_for_expression;

/// Fast identifier validation using bytes
#[inline]
pub fn is_valid_identifier_fast(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let first = bytes[0];
    if !first.is_ascii_alphabetic() && first != b'_' && first != b'$' {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$')
}
