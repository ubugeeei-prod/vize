use super::condense::{condense_internal, is_vue_ws};
use vize_l0::Allocator;

/// The parse-time text view of a whitespace-only conditional gap, for authored
/// surface consumers. Compiler regions drop these gaps regardless of this view.
pub fn branch_gap_text<'a>(
    allocator: &'a Allocator,
    text: &'a str,
    in_pre: bool,
) -> Option<&'a str> {
    if in_pre {
        Some(text)
    } else if text.chars().all(is_vue_ws) {
        (!text.contains(['\n', '\r'])).then_some(" ")
    } else {
        Some(condense_internal(allocator, text).unwrap_or(text))
    }
}
