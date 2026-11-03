//! Vue's template-expression globals, separate from host JavaScript scopes.
//!
//! A name may exist on the browser/Node host while still resolving through the
//! component proxy in a Vue template. Keep this compiler policy independent of
//! Croquis' broader semantic global inventory.
//!
//! Reference: vuejs/core v3.5.41 packages/shared/src/globalsAllowList.ts.

/// Whether Vue leaves an authored template global unprefixed.
#[inline]
pub fn is_vue_template_global(name: &str) -> bool {
    matches!(
        name,
        "Infinity"
            | "undefined"
            | "NaN"
            | "isFinite"
            | "isNaN"
            | "parseFloat"
            | "parseInt"
            | "decodeURI"
            | "decodeURIComponent"
            | "encodeURI"
            | "encodeURIComponent"
            | "Math"
            | "Number"
            | "Date"
            | "Array"
            | "Object"
            | "Boolean"
            | "String"
            | "RegExp"
            | "Map"
            | "Set"
            | "JSON"
            | "Intl"
            | "BigInt"
            | "console"
            | "Error"
            | "Symbol"
    )
}

#[cfg(test)]
mod tests {
    use super::is_vue_template_global;

    #[test]
    fn host_globals_do_not_extend_the_vue_template_allowlist() {
        for name in [
            "document",
            "window",
            "navigator",
            "globalThis",
            "Promise",
            "Proxy",
            "Function",
            "Reflect",
            "WeakMap",
            "WeakSet",
            "module",
            "exports",
            "TypeError",
            "Uint8Array",
        ] {
            assert!(!is_vue_template_global(name), "{name}");
        }
        for name in ["Math", "Intl", "Array", "console", "undefined", "Symbol"] {
            assert!(is_vue_template_global(name), "{name}");
        }
    }
}
