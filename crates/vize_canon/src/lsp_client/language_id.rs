//! Canonical LSP language identifiers for TypeScript project sources.

/// Return the language identifier matching the document's authored extension.
///
/// TSGo uses this value to choose the document's `ScriptKind` when an overlay
/// is first opened. Reporting JavaScript as TypeScript disables JavaScript's
/// JSDoc checking even when the project enables `allowJs` and `checkJs`.
pub(super) fn for_uri(uri: &str) -> &'static str {
    if uri.ends_with(".jsx") {
        "javascriptreact"
    } else if uri.ends_with(".js") || uri.ends_with(".mjs") || uri.ends_with(".cjs") {
        "javascript"
    } else if uri.ends_with(".tsx") {
        "typescriptreact"
    } else {
        "typescript"
    }
}

#[cfg(test)]
mod tests {
    use super::for_uri;

    #[test]
    fn preserves_the_authored_script_family_for_every_project_extension() {
        for (uri, expected) in [
            ("file:///project/source.js", "javascript"),
            ("file:///project/source.mjs", "javascript"),
            ("file:///project/source.cjs", "javascript"),
            ("file:///project/source.jsx", "javascriptreact"),
            ("file:///project/source.ts", "typescript"),
            ("file:///project/source.mts", "typescript"),
            ("file:///project/source.cts", "typescript"),
            ("file:///project/source.tsx", "typescriptreact"),
            ("file:///project/App.vue.ts", "typescript"),
            ("file:///project/App.vue.tsx", "typescriptreact"),
        ] {
            assert_eq!(for_uri(uri), expected, "{uri}");
        }
    }
}
