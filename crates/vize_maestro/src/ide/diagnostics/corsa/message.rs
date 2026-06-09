//! Vue-flavored rewriting of raw Corsa/TypeScript diagnostic messages.

use vize_carton::cstr;

/// Rewrite a Corsa diagnostic message with a Vue-flavored hint when the
/// raw TypeScript phrasing has a more actionable Vue interpretation.
///
/// The original wording is preserved as the prefix so the user can still see
/// what TypeScript reported. The added hint points at the most common Vue
/// cause for that error shape.
pub(super) fn rewrite_corsa_message(message: &str) -> String {
    if let Some(prop) = property_does_not_exist_property(message)
        && prop != "value"
    {
        return cstr!(
            "{message}\n\nIf you intended to read the reactive value, try `.value`. (vize/types)"
        )
        .into();
    }
    if message.starts_with("Type 'Ref<") && message.contains("is not assignable to type") {
        return cstr!(
            "{message}\n\nDid you forget `.value`? Vue refs need to be unwrapped in script context. (vize/types)"
        ).into();
    }
    message.to_string()
}

/// Extract the property name from a TS7053/TS2339 "Property 'X' does not
/// exist on type 'Y'" message. Returns `None` for unrelated messages.
fn property_does_not_exist_property(message: &str) -> Option<&str> {
    let head = "Property '";
    let after = message.strip_prefix(head)?;
    let end = after.find('\'')?;
    let rest = &after[end..];
    if !rest.starts_with("' does not exist") {
        return None;
    }
    Some(&after[..end])
}

#[cfg(test)]
mod hint_tests {
    use super::{property_does_not_exist_property, rewrite_corsa_message};

    #[test]
    fn rewrites_property_does_not_exist_with_value_hint() {
        let original = "Property 'toFixed' does not exist on type 'Ref<number>'.";
        let rewritten = rewrite_corsa_message(original);
        assert!(rewritten.contains(original));
        assert!(
            rewritten.contains(".value"),
            "expected a .value hint, got {rewritten:?}"
        );
    }

    #[test]
    fn leaves_known_property_value_alone() {
        // We don't want to suggest `.value` on a `.value` access — that's
        // already what the user wrote.
        let original = "Property 'value' does not exist on type 'unknown'.";
        let rewritten = rewrite_corsa_message(original);
        assert_eq!(rewritten, original);
    }

    #[test]
    fn rewrites_ref_assignment_with_unwrap_hint() {
        let original = "Type 'Ref<number>' is not assignable to type 'number'.";
        let rewritten = rewrite_corsa_message(original);
        assert!(rewritten.contains(original));
        assert!(rewritten.contains("Did you forget `.value`"));
    }

    #[test]
    fn passes_through_unrelated_messages() {
        let original = "Expected 1 argument, but got 0.";
        assert_eq!(rewrite_corsa_message(original), original);
    }

    #[test]
    fn property_extractor_returns_name() {
        assert_eq!(
            property_does_not_exist_property("Property 'foo' does not exist on type 'Bar'."),
            Some("foo")
        );
        assert_eq!(
            property_does_not_exist_property("Cannot find name 'foo'."),
            None
        );
    }
}
