//! Public names for Vize's private Vue helper types in diagnostic text.
//!
//! The generated code types `defineProps<T>()` through private copies of Vue's
//! own aliases, `__DefineProps<__LooseRequired<T>, K>`, so that Vue 2.7 and a
//! project's own `DefineProps` import can never collide with them. The checker
//! prints the alias name it sees, and a user would then read a name they can
//! neither import nor look up. Each private alias here has the same parameters
//! and meaning as the Vue type it shadows, so the message is reworded to the
//! name Vue (and `vue-tsc`) uses.

use vize_carton::{String, ToCompactString};

/// Private helper alias and the Vue type it is a copy of.
const PUBLIC_NAMES: [(&str, &str); 2] = [
    ("__DefineProps", "DefineProps"),
    ("__LooseRequired", "LooseRequired"),
];

/// Reword every private helper alias in `message` to its public Vue name.
/// Only a whole identifier applied to type arguments (`__DefineProps<`) is a
/// reference to the alias.
pub fn present_internal_type_names(message: &str) -> String {
    if !message.contains("__") {
        return message.to_compact_string();
    }
    let mut presented = message.to_compact_string();
    for (private, public) in PUBLIC_NAMES {
        if presented.contains(private) {
            presented = replace_type_reference(&presented, private, public);
        }
    }
    presented
}

fn replace_type_reference(message: &str, private: &str, public: &str) -> String {
    let mut rewritten = String::with_capacity(message.len());
    let mut rest = message;
    let mut preceding: Option<char> = None;
    while let Some((before, after)) = rest.split_once(private) {
        rewritten.push_str(before);
        let boundary = before
            .chars()
            .next_back()
            .or(preceding)
            .is_none_or(|character| !is_identifier_character(character));
        let applied = after.starts_with('<');
        rewritten.push_str(if boundary && applied { public } else { private });
        preceding = private.chars().next_back();
        rest = after;
    }
    rewritten.push_str(rest);
    rewritten
}

fn is_identifier_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_' || character == '$'
}

#[cfg(test)]
mod tests {
    use super::present_internal_type_names;

    #[test]
    fn define_props_reads_as_vue_names_it() {
        assert_eq!(
            present_internal_type_names(
                "Type '__DefineProps<__LooseRequired<Props>, \"disabled\">' is not assignable to type 'number'."
            ),
            "Type 'DefineProps<LooseRequired<Props>, \"disabled\">' is not assignable to type 'number'."
        );
    }

    #[test]
    fn only_whole_applied_references_are_reworded() {
        for message in [
            "Cannot find name '__DefineProps'.",
            "Property 'my__DefineProps' does not exist.",
            "Type 'x__LooseRequired<T>' is not assignable.",
        ] {
            assert_eq!(present_internal_type_names(message), message);
        }
    }
}
