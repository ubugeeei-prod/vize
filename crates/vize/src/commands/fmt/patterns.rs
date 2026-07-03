pub(super) const FORMAT_EXTENSIONS: &[&str] = &[
    "vue", "js", "mjs", "cjs", "ts", "mts", "cts", "jsx", "tsx", "json", "jsonc", "yaml", "yml",
    "md", "markdown",
];

pub(super) const FORMAT_EXTENSIONS_DISPLAY: &str = ".vue, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, .tsx, .json, .jsonc, .yaml, .yml, .md, or .markdown";

#[allow(clippy::disallowed_types)]
pub(super) fn default_fmt_patterns() -> Vec<std::string::String> {
    FORMAT_EXTENSIONS
        .iter()
        .map(|extension| format!("./**/*.{extension}"))
        .collect()
}

#[inline]
#[allow(clippy::disallowed_types)]
pub(super) fn has_explicit_patterns(patterns: &[std::string::String]) -> bool {
    patterns != default_fmt_patterns().as_slice()
}

#[inline]
pub(super) fn is_format_extension(extension: &str) -> bool {
    FORMAT_EXTENSIONS.contains(&extension)
}

#[cfg(test)]
mod tests {
    use super::{FORMAT_EXTENSIONS, FORMAT_EXTENSIONS_DISPLAY, default_fmt_patterns};

    #[test]
    fn default_patterns_cover_each_format_extension_once() {
        let expected = FORMAT_EXTENSIONS
            .iter()
            .map(|extension| format!("./**/*.{extension}"))
            .collect::<Vec<_>>();

        assert_eq!(default_fmt_patterns(), expected);
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
