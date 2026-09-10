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

pub(super) fn unmatched_lint_patterns_message(patterns: &[vize_s0::String]) -> vize_s0::String {
    vize_s0::cstr!(
        "Warning: no {LINT_EXTENSIONS_DISPLAY} files found matching patterns: {patterns:?}"
    )
}

pub(super) fn write_no_files(format: vize_patina::OutputFormat, patterns: &[vize_s0::String]) {
    eprintln!("{}", no_lint_files_message(patterns));
    if format == vize_patina::OutputFormat::Json {
        super::stdout::write(vize_patina::format_results(&[], &[], format).as_bytes());
    }
}

pub(super) fn write_unmatched_patterns(patterns: &[vize_s0::String]) {
    eprintln!("{}", unmatched_lint_patterns_message(patterns));
}

pub(super) fn write_unmatched_explicit_patterns(
    input_patterns: &[vize_s0::String],
    unmatched_patterns: &[vize_s0::String],
) -> usize {
    if !has_explicit_patterns(input_patterns) || unmatched_patterns.is_empty() {
        return 0;
    }
    write_unmatched_patterns(unmatched_patterns);
    unmatched_patterns.len()
}

#[inline]
pub(super) fn has_explicit_patterns(patterns: &[vize_s0::String]) -> bool {
    patterns.len() != LINT_DEFAULT_PATTERNS.len()
        || patterns
            .iter()
            .zip(LINT_DEFAULT_PATTERNS)
            .any(|(actual, expected)| actual.as_str() != *expected)
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
    use super::{
        LINT_DEFAULT_PATTERNS, LINT_EXTENSIONS, LINT_EXTENSIONS_DISPLAY, has_explicit_patterns,
        unmatched_lint_patterns_message,
    };

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

    #[test]
    fn default_lint_patterns_are_not_explicit() {
        let patterns = LINT_DEFAULT_PATTERNS
            .iter()
            .map(|pattern| vize_s0::String::from(*pattern))
            .collect::<Vec<_>>();

        assert!(!has_explicit_patterns(&patterns));
        assert!(has_explicit_patterns(&[vize_s0::String::from(
            "src/**/*.vue"
        )]));
    }

    #[test]
    fn unmatched_pattern_message_lists_every_empty_input() {
        let output = unmatched_lint_patterns_message(&[
            vize_s0::String::from("src/**/*.{vue,ts}"),
            vize_s0::String::from("packages/*/missing.vue"),
        ]);

        assert_eq!(
            output,
            "Warning: no .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx files found matching patterns: [\"src/**/*.{vue,ts}\", \"packages/*/missing.vue\"]"
        );
    }
}
