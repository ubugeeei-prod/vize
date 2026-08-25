pub(super) const LINT_EXTENSIONS: &[&str] = &[
    "vue", "html", "htm", "js", "mjs", "cjs", "ts", "mts", "cts", "jsx", "tsx",
];

pub(super) const LINT_DEFAULT_PATTERNS: &[&str] = &[
    "./**/*.vue",
    "./**/*.html",
    "./**/*.htm",
    "./**/*.js",
    "./**/*.mjs",
    "./**/*.cjs",
    "./**/*.ts",
    "./**/*.mts",
    "./**/*.cts",
    "./**/*.jsx",
    "./**/*.tsx",
];

pub(super) const LINT_EXTENSIONS_DISPLAY: &str =
    ".vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx";

pub(super) fn no_lint_files_message(patterns: &[vize_s0::String]) -> vize_s0::String {
    vize_s0::cstr!("No {LINT_EXTENSIONS_DISPLAY} files found matching patterns: {patterns:?}")
}

pub(super) fn write_no_files(format: vize_patina::OutputFormat, patterns: &[vize_s0::String]) {
    eprintln!("{}", no_lint_files_message(patterns));
    if format == vize_patina::OutputFormat::Json {
        super::stdout::write(vize_patina::format_results(&[], &[], format).as_bytes());
    }
}

#[inline]
pub(super) fn is_lint_extension(extension: &str) -> bool {
    LINT_EXTENSIONS.contains(&extension)
}

#[inline]
pub(super) fn is_standalone_html_extension(extension: &str) -> bool {
    matches!(extension, "html" | "htm")
}

#[inline]
pub(super) fn is_plain_script_extension(extension: &str) -> bool {
    matches!(extension, "js" | "mjs" | "cjs" | "ts" | "mts" | "cts")
}

#[cfg(test)]
mod tests {
    use super::{LINT_DEFAULT_PATTERNS, LINT_EXTENSIONS, LINT_EXTENSIONS_DISPLAY};

    #[test]
    fn default_patterns_cover_each_lint_extension_once() {
        let expected = LINT_EXTENSIONS
            .iter()
            .map(|extension| format!("./**/*.{extension}"))
            .collect::<Vec<_>>();

        assert_eq!(LINT_DEFAULT_PATTERNS, expected.as_slice());
    }

    #[test]
    fn display_text_mentions_each_lint_extension() {
        let display_tokens = LINT_EXTENSIONS_DISPLAY
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>();
        for extension in LINT_EXTENSIONS {
            assert!(
                display_tokens.contains(extension),
                "missing .{extension} from display text"
            );
        }
    }
}
