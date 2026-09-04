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

/// The root scope `vize_croquis::ScopeChain::new` seeds with
/// `JS_UNIVERSAL_GLOBALS`. `TransformContext::is_in_scope` answers `true`
/// for every one of them, so the shipped transform never prefixes them —
/// a *wider* set than [`is_global_allowed`], and the difference is
/// visible output (`Intl.DateTimeFormat()` stays bare). Only the
/// transform-side decision consults this; the codegen visitor keeps to
/// the allowlist, exactly as the shipped lane does.
pub(super) fn is_scope_chain_global(name: &str) -> bool {
    if is_global_allowed(name) {
        return true;
    }
    matches!(name, |"AggregateError"| "ArrayBuffer"
        | "AsyncFunction"
        | "AsyncGenerator"
        | "AsyncGeneratorFunction"
        | "AsyncIterator"
        | "Atomics"
        | "BigInt64Array"
        | "BigUint64Array"
        | "DataView"
        | "EvalError"
        | "Float32Array"
        | "Float64Array"
        | "Generator"
        | "GeneratorFunction"
        | "Int16Array"
        | "Int32Array"
        | "Int8Array"
        | "Intl"
        | "Iterator"
        | "RangeError"
        | "ReferenceError"
        | "SharedArrayBuffer"
        | "SyntaxError"
        | "TypeError"
        | "URIError"
        | "Uint16Array"
        | "Uint32Array"
        | "Uint8Array"
        | "Uint8ClampedArray"
        | "eval"
        | "this")
}

#[cfg(test)]
mod scope_chain_tests {
    use super::{is_global_allowed, is_scope_chain_global};

    #[test]
    fn the_seeded_scope_is_wider_than_the_allowlist() {
        assert!(is_scope_chain_global("Intl"));
        assert!(is_scope_chain_global("TypeError"));
        assert!(is_scope_chain_global("Uint8Array"));
        assert!(!is_global_allowed("Intl"));
        // Everything the allowlist admits is seeded too.
        assert!(is_scope_chain_global("Math"));
        assert!(is_scope_chain_global("$event"));
        assert!(!is_scope_chain_global("Zork"));
    }
}
