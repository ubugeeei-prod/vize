//! CSS formatting using lightningcss.
//!
//! This module provides formatting for CSS/SCSS/Less content
//! in Vue SFC `<style>` blocks using lightningcss for parsing and printing.

mod comment_scan;
mod stabilization;

use crate::error::FormatError;
use crate::options::FormatOptions;
use comment_scan::{SegmentKind, has_nested_comment, split_top_level_comments};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use vize_s0::{String, ToCompactString};

/// Format CSS content using lightningcss.
///
/// Top-level (depth 0) comments are extracted before parsing and re-inserted
/// at their original boundaries, because lightningcss drops non-license
/// comments during parse. Nested comments keep the original block text instead
/// of risking silent data loss.
pub fn format_style_content(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(String::default());
    }

    if has_nested_comment(source) {
        return Ok(trimmed.to_compact_string());
    }

    if !contains_comment(source) {
        return format_chunk(trimmed, options);
    }

    format_with_preserved_top_level_comments(source, options)
}

fn format_with_preserved_top_level_comments(
    source: &str,
    options: &FormatOptions,
) -> Result<String, FormatError> {
    let newline = options.newline_string();
    let mut output: String = String::with_capacity(source.len());
    let mut emitted_any = false;

    for segment in split_top_level_comments(source) {
        match segment.kind {
            SegmentKind::Code => {
                let trimmed_chunk = segment.content.trim();
                if trimmed_chunk.is_empty() {
                    continue;
                }
                let formatted = format_chunk(trimmed_chunk, options)?;
                let formatted = formatted
                    .as_str()
                    .trim_end_matches('\n')
                    .trim_end_matches('\r');
                if formatted.is_empty() {
                    continue;
                }
                if emitted_any {
                    output.push_str(newline);
                }
                output.push_str(formatted);
                emitted_any = true;
            }
            SegmentKind::Comment => {
                if emitted_any {
                    output.push_str(newline);
                }
                output.push_str(segment.content);
                emitted_any = true;
            }
        }
    }

    Ok(output)
}

fn format_chunk(trimmed: &str, options: &FormatOptions) -> Result<String, FormatError> {
    stabilization::format_to_fixed_point(trimmed, |source| format_chunk_once(source, options))
}

fn format_chunk_once(trimmed: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let stylesheet = StyleSheet::parse(trimmed, ParserOptions::default())
        .map_err(|e| FormatError::StyleFormatError(e.to_compact_string()))?;

    let indent_width = options.tab_width;
    let printer_options = PrinterOptions {
        minify: false,
        ..Default::default()
    };

    let result = stylesheet
        .to_css(printer_options)
        .map_err(|e| FormatError::StyleFormatError(e.to_compact_string()))?;

    let mut code: String = result.code.into();

    // lightningcss uses 2-space indent by default; re-indent if needed
    if options.use_tabs || indent_width != 2 {
        code = reindent_css(&code, options);
    }

    Ok(code)
}

/// Re-indent CSS output to match the configured indent style
fn reindent_css(source: &str, options: &FormatOptions) -> String {
    let indent = options.indent_string();
    let newline = options.newline_string();
    let mut result: String = String::with_capacity(source.len());

    for line in source.lines() {
        // Count leading spaces (lightningcss uses 2-space indent)
        let leading_spaces = line.len() - line.trim_start().len();
        let indent_level = leading_spaces / 2;
        let trimmed = line.trim_start();

        if trimmed.is_empty() {
            result.push_str(newline);
            continue;
        }

        for _ in 0..indent_level {
            result.push_str(&indent);
        }
        result.push_str(trimmed);
        result.push_str(newline);
    }

    // Remove trailing newline added by the loop
    if result.ends_with(newline) {
        result.truncate(result.len() - newline.len());
    }

    result
}

fn contains_comment(source: &str) -> bool {
    memchr::memmem::find(source.as_bytes(), b"/*").is_some()
}

#[cfg(test)]
mod tests {
    use super::{FormatOptions, format_style_content};

    #[test]
    fn test_background_position_shorthand_reaches_fixed_point_in_one_pass() {
        // `background-position: left 1em top 50%` is a non-idempotent case for
        // lightningcss: it first prints `1em 50%`, then a re-parse collapses
        // the redundant center `50%` to `1em`. The formatter must reach that
        // normal form in a single `vize fmt` pass. (#3248)
        let source = ".a { background-position: left 1em top 50%; }";
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();
        assert_eq!(result.as_str(), ".a {\n  background-position: 1em;\n}\n");

        // And formatting the result again is a no-op.
        let again = format_style_content(&result, &options).unwrap();
        assert_eq!(again, result);
    }

    #[test]
    fn test_format_simple_css() {
        let source = ".container{color:red;display:flex;gap:8px}";
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_nested_css_at_rule() {
        let source = "@media (min-width: 640px){.container{color:red}}";
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_empty_css() {
        let source = "";
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_css_whitespace_only() {
        let source = "   \n\t  ";
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_format_preserves_top_level_block_comment_between_rules() {
        let source = concat!(
            "/* stylelint-disable-next-line selector-id-pattern */\n",
            "#legacy-id { display: grid; }\n",
        );
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();

        assert!(
            result.contains("/* stylelint-disable-next-line selector-id-pattern */"),
            "top-level CSS comments must survive formatting; got: {result}",
        );
        assert!(
            result.contains("#legacy-id {"),
            "rule should still be formatted; got: {result}",
        );
    }

    #[test]
    fn test_format_preserves_multiple_top_level_block_comments() {
        let source = concat!(
            "/* NOTE: Avoid using kebab-case for better readability. */\n",
            ".foo { color: red; }\n",
            "/* trailing note */\n",
        );
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();

        assert!(result.contains("/* NOTE: Avoid using kebab-case for better readability. */"));
        assert!(result.contains("/* trailing note */"));
        assert!(result.contains(".foo {"));
    }

    #[test]
    fn test_format_preserves_nested_block_comments_by_leaving_block_raw() {
        let source = concat!(
            ".box {\n",
            "  /* keep color note */\n",
            "  color: red;\n",
            "  /* keep nested note */\n",
            "  .inner {\n",
            "    color: blue; /* keep inline note */\n",
            "  }\n",
            "}\n",
        );
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();

        assert_eq!(result.as_str(), source.trim());
        assert!(result.contains("/* keep color note */"));
        assert!(result.contains("/* keep nested note */"));
        assert!(result.contains("/* keep inline note */"));
    }

    #[test]
    fn test_format_keeps_comment_like_content_inside_strings_as_string() {
        let source = ".x { content: \"/* not a comment */\"; }";
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();
        // The content stays as string literal, no segment split happens, so
        // lightningcss emits a clean single-rule output.
        assert!(result.contains(".x"));
        assert!(result.contains("\"/* not a comment */\""));
    }

    #[test]
    fn test_format_keeps_comment_markers_inside_unquoted_urls() {
        let source = concat!(
            ".asset{background:url(https://example.test/a/*/icon.svg);color:red}\n",
            "/* after */",
        );
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();

        assert!(result.contains(".asset {\n"));
        assert!(result.contains("https://example.test/a/*/icon.svg"));
        assert!(result.contains("/* after */"));
    }

    #[test]
    fn test_format_splits_top_level_comments_after_import_url_data() {
        let source = concat!(
            "@import url(https://example.test/a/*/reset.css);\n",
            "/* import note */\n",
            ".asset{color:red}",
        );
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();

        assert!(result.contains("https://example.test/a/*/reset.css"));
        assert!(result.contains("/* import note */"));
        assert!(result.contains(".asset {\n"));
    }

    #[test]
    fn test_format_charset_before_top_level_comment_reaches_fixed_point() {
        let source = concat!(
            "@charset \"UTF-8\";\n",
            "/* comment */\n",
            ".a {\n",
            "  color: red;\n",
            "}",
        );
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();
        let again = format_style_content(result.as_str(), &options).unwrap();

        assert_eq!(result, again);
        assert!(result.as_str().starts_with("/* comment */\n.a {"));
    }
}
