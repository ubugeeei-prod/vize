//! The identifier allowlist the shipped prefixer never prefixes: JS
//! globals, render-function locals, and the `_toNumber` helper. Conditional
//! event locals are introduced by the handler scope, never globally allowed.
//! This stage cannot depend on Croquis; its policy matches the legacy
//! emitter's `is_template_global` boundary.

/// Names available independently of a lexical binding.
pub(crate) fn is_global_allowed(name: &str) -> bool {
    vize_l0::template_globals::is_vue_template_global(name)
        || matches!(
            name,
            "require"
                | "arguments"
                | "import"
                | "_ctx"
                | "_cache"
                | "_push"
                | "_parent"
                | "_toNumber"
        )
}

/// An unprefixed global whose value is stable for constant folding and hoisting.
pub(crate) fn is_constant_global(name: &str) -> bool {
    vize_l0::template_globals::is_vue_template_global(name)
}

pub(super) fn is_generated_filter_helper(name: &str) -> bool {
    name.strip_prefix("_filter_")
        .is_some_and(|suffix| !suffix.is_empty())
}

/// `vize_atelier_core::steps::is_simple_identifier`: Unicode-alphabetic
/// start, alphanumeric continuation, `_` and `$` allowed everywhere.
pub(crate) fn is_simple_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

#[cfg(test)]
mod tests {
    use super::{is_global_allowed, is_simple_identifier};

    #[test]
    fn globals_match_the_shipped_allowlist() {
        assert!(is_global_allowed("Array"));
        assert!(!is_global_allowed("$event"));
        assert!(is_global_allowed("_toNumber"));
        assert!(!is_global_allowed("myVar"));
        assert!(!is_global_allowed("$slots"));
    }

    #[test]
    fn simple_identifier_matches_the_shipped_rule() {
        assert!(is_simple_identifier("foo"));
        assert!(is_simple_identifier("_bar"));
        assert!(is_simple_identifier("$baz"));
        assert!(is_simple_identifier("foo123"));
        assert!(is_simple_identifier("名前"));
        assert!(!is_simple_identifier("123foo"));
        assert!(!is_simple_identifier("foo-bar"));
        assert!(!is_simple_identifier("foo.bar"));
        assert!(!is_simple_identifier(" foo"));
        assert!(!is_simple_identifier(""));
    }
}

/// The transform consults lexical template bindings separately from globals.
/// The semantic JS universal scope must not widen Vue's compiler allow-list.
pub(super) fn is_scope_chain_global(name: &str) -> bool {
    is_global_allowed(name) || name == "this"
}

#[cfg(test)]
mod scope_chain_tests {
    use super::{is_global_allowed, is_scope_chain_global};

    #[test]
    fn the_transform_and_codegen_use_the_same_vue_global_boundary() {
        for name in ["Math", "Intl", "require", "arguments"] {
            assert!(is_scope_chain_global(name), "{name}");
            assert!(is_global_allowed(name), "{name}");
        }
        for name in [
            "document",
            "globalThis",
            "Promise",
            "TypeError",
            "Uint8Array",
        ] {
            assert!(!is_scope_chain_global(name), "{name}");
            assert!(!is_global_allowed(name), "{name}");
        }
    }
}
