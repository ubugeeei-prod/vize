//! The identifier allowlist the shipped prefixer never prefixes: JS
//! globals, render-function locals, `$event`, and the `_toNumber` helper
//! (`vize_croquis::builtins::GLOBAL_ALLOWLIST_SET`), copied because this
//! crate cannot depend on `vize_croquis`; byte-identical output is the
//! P2-11 bar, so the two lists must stay identical.

/// `vize_croquis::builtins::is_global_allowed`.
pub(crate) fn is_global_allowed(name: &str) -> bool {
    matches!(
        name,
        "Infinity"
            | "undefined"
            | "NaN"
            | "Array"
            | "Boolean"
            | "Date"
            | "Error"
            | "Function"
            | "JSON"
            | "Math"
            | "Number"
            | "Object"
            | "Promise"
            | "Proxy"
            | "Reflect"
            | "RegExp"
            | "Set"
            | "String"
            | "Symbol"
            | "Map"
            | "WeakMap"
            | "WeakSet"
            | "BigInt"
            | "parseInt"
            | "parseFloat"
            | "isNaN"
            | "isFinite"
            | "decodeURI"
            | "decodeURIComponent"
            | "encodeURI"
            | "encodeURIComponent"
            | "arguments"
            | "console"
            | "window"
            | "document"
            | "navigator"
            | "globalThis"
            | "require"
            | "import"
            | "exports"
            | "module"
            | "_ctx"
            | "_cache"
            | "_push"
            | "_parent"
            | "$event"
            | "_toNumber"
    )
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
        assert!(is_global_allowed("$event"));
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
