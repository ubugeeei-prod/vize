//! Project source-owned parameter facts, without reparsing or guessing names.

use vize_l0::Span;
use vize_l2::scope::{ScopeFacts, ScopeOrigin};

/// Exceptional alias names already observed by the original binding-goal
/// producer. `Some(empty)` is a refusal, never permission to scan raw text.
/// Ordinary aliases keep the existing consumer path and allocation window.
#[doc(hidden)]
pub fn exceptional_for_binding_names<'a>(
    alias: &str,
    alias_span: Span,
    facts: Option<&'a ScopeFacts>,
) -> Option<impl Iterator<Item = &'a str> + use<'a>> {
    if alias.is_ascii() && !alias.contains('\\') {
        return None;
    }
    Some(
        facts
            .into_iter()
            .flat_map(|facts| &facts.bindings)
            .filter_map(move |binding| match binding.origin {
                ScopeOrigin::Authored { span }
                    if span.start >= alias_span.start && span.end <= alias_span.end =>
                {
                    Some(binding.name.as_str())
                }
                _ => None,
            }),
    )
}
