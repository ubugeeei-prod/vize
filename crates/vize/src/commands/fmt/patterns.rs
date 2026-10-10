pub(super) const FORMAT_EXTENSIONS: &[&str] = &[
    "vue", "js", "mjs", "cjs", "ts", "mts", "cts", "jsx", "tsx", "json", "jsonc", "yaml", "yml",
    "md", "markdown",
];

pub(super) const FORMAT_EXTENSIONS_DISPLAY: &str = ".vue, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, .tsx, .json, .jsonc, .yaml, .yml, .md, or .markdown";

pub(super) fn is_unimplemented_document_extension(extension: &str) -> bool {
    matches!(extension, "yaml" | "yml" | "md" | "markdown")
}

#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
pub(super) fn default_fmt_patterns() -> Vec<std::string::String> {
    FORMAT_EXTENSIONS
        .iter()
        .filter(|extension| !is_unimplemented_document_extension(extension))
        .map(|extension| vize_l0::cstr!("./**/*.{extension}").into())
        .collect()
}

#[inline]
pub(super) fn is_format_extension(extension: &str) -> bool {
    FORMAT_EXTENSIONS.contains(&extension)
}

#[cfg(test)]
mod tests {
    use super::{FORMAT_EXTENSIONS, FORMAT_EXTENSIONS_DISPLAY, default_fmt_patterns};

    #[test]
    fn default_patterns_cover_only_implemented_format_extensions() {
        let expected = [
            "vue", "js", "mjs", "cjs", "ts", "mts", "cts", "jsx", "tsx", "json", "jsonc",
        ]
        .iter()
        .map(|extension| vize_l0::cstr!("./**/*.{extension}"))
        .collect::<Vec<_>>();

        assert_eq!(default_fmt_patterns(), expected);
        for extension in ["yaml", "yml", "md", "markdown"] {
            assert!(super::is_unimplemented_document_extension(extension));
        }
    }

    #[test]
    fn display_text_mentions_each_format_extension() {
        let display_tokens = FORMAT_EXTENSIONS_DISPLAY
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>();
        for extension in FORMAT_EXTENSIONS {
            assert!(
                display_tokens.contains(extension),
                "missing .{extension} from display text"
            );
        }
    }
}
