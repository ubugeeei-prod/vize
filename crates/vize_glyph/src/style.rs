//! CSS formatting using lightningcss.
//!
//! This module provides formatting for CSS/SCSS/Less content
//! in Vue SFC `<style>` blocks using lightningcss for parsing and printing.

mod authored;
mod color;
mod comment_scan;
mod number;
mod stabilization;

use crate::error::FormatError;
use crate::options::FormatOptions;
use comment_scan::{SegmentKind, has_nested_comment, split_top_level_comments};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use vize_l0::{String, ToCompactString};

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
    let mut separator_lines = 1;

    for segment in split_top_level_comments(source) {
        match segment.kind {
            SegmentKind::Code => {
                let trimmed_chunk = segment.content.trim();
                if trimmed_chunk.is_empty() {
                    separator_lines =
                        separator_lines.max(authored::source_separator_lines(segment.content));
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
                    for _ in 0..separator_lines
                        .max(authored::source_separator_lines_before(segment.content))
                    {
                        output.push_str(newline);
                    }
                }
                output.push_str(formatted);
                emitted_any = true;
                separator_lines = authored::source_separator_lines_after(segment.content);
            }
            SegmentKind::Comment => {
                if emitted_any {
                    for _ in 0..separator_lines {
                        output.push_str(newline);
                    }
                }
                output.push_str(segment.content);
                emitted_any = true;
                separator_lines = 1;
            }
        }
    }

    Ok(output)
}

fn format_chunk(trimmed: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let colors = color::protect(trimmed);
    let formatted = stabilization::format_to_fixed_point(colors.source.as_str(), |source| {
        format_chunk_once(source, options)
    })?;
    let formatted = colors.restore(formatted);
    // The CSS printer also performs syntax and value normalization. A formatter
    // must never silently change browser support or the scoped selector target.
    // Format only structural whitespace when the print changes authored CSS.
    if authored::changes_authored_css(trimmed, formatted.as_str()) {
        Ok(authored::format_layout_only(trimmed, options))
    } else {
        Ok(formatted)
    }
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

    // lightningcss omits leading zeroes even with minify disabled; Oxfmt keeps
    // them in standalone CSS, so align the style block's printed number tokens.
    let mut code = number::add_leading_zero_to_fractional_numbers(&result.code);

    // lightningcss uses 2-space indent by default; re-indent if needed
    if options.use_tabs || indent_width != 2 {
        code = authored::reindent_css(&code, options);
    }

    Ok(code)
}

fn contains_comment(source: &str) -> bool {
    memchr::memmem::find(source.as_bytes(), b"/*").is_some()
}

#[cfg(test)]
mod tests {
    use super::{FormatOptions, format_style_content};

    #[test]
    fn test_background_position_shorthand_stays_authored_and_stable() {
        // lightningcss normalizes this shorthand over multiple passes. Keep
        // the original declaration so formatting remains syntax preserving.
        let source = ".a { background-position: left 1em top 50%; }";
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();
        assert!(result.contains("background-position: left 1em top 50%;"));

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
    fn test_style_numbers_match_standalone_css_leading_zeroes() {
        let source =
            ".sample { opacity: 0.5; transition: opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1); }";
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();
        assert!(result.contains("opacity: 0.5;"), "{result}");
        assert!(
            result.contains("opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1)"),
            "{result}",
        );
        assert_eq!(format_style_content(&result, &options).unwrap(), result);
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

        assert!(result.contains(".asset {"));
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
        assert!(
            result
                .as_str()
                .starts_with("@charset \"UTF-8\";\n/* comment */\n.a {")
        );
    }
}
