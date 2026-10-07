//! CSS formatting using lightningcss.
//!
//! This module provides formatting for CSS/SCSS/Less content
//! in Vue SFC `<style>` blocks using lightningcss for parsing and printing.

mod authored;
mod blank_lines;
#[path = "style_chunk.rs"]
mod chunk;
mod color;
mod comment_scan;
mod declaration;
mod number;
mod rule_layout;
mod stabilization;
mod values;

use chunk::{contains_comment, format_chunk};

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
    if options.end_of_line == crate::EndOfLine::Auto {
        return options.format_with_source_line_ending(source, |options| {
            format_style_content(source, options)
        });
    }
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
    fn style_block_keeps_box_values_and_implicit_nested_selectors() {
        // #7049: formatting may change whitespace, indentation, line breaks,
        // and quotes, but not declaration values or selectors.
        let source = concat!(
            "<template>\n",
            "  <hr />\n",
            "</template>\n",
            "\n",
            "<style scoped>\n",
            "hr {\n",
            "  border: solid;\n",
            "  border-width: thin 0 0 0;\n",
            "  margin: 0 0 0 0;\n",
            "}\n",
            "\n",
            ".wrap {\n",
            "  color: red;\n",
            "\n",
            "  .item {\n",
            "    color: blue;\n",
            "  }\n",
            "}\n",
            "</style>\n",
        );
        let options = FormatOptions::default();
        let formatted = crate::format_sfc(source, &options).unwrap().code;
        assert!(
            formatted.contains("border-width: thin 0 0 0;"),
            "{formatted}"
        );
        assert!(formatted.contains("margin: 0 0 0 0;"), "{formatted}");
        assert!(
            formatted.contains("\n  .item {"),
            "nested selector must stay authored and indented: {formatted}"
        );
        assert!(
            !formatted.contains('&'),
            "nested selectors must not gain a nesting prefix: {formatted}"
        );
        assert_eq!(
            crate::format_sfc(&formatted, &options).unwrap().code,
            formatted
        );

        let compact = concat!(
            "hr{border:solid;border-width:thin 0 0 0;margin:0 0 0 0}",
            ".wrap{color:red;.item{color:blue}}",
        );
        let pretty = format_style_content(compact, &options).unwrap();
        assert!(pretty.contains("thin 0 0 0"), "{pretty}");
        assert!(pretty.contains("0 0 0 0"), "{pretty}");
        assert!(pretty.contains("\n  .item"), "{pretty}");
        assert!(!pretty.contains('&'), "{pretty}");
        assert_eq!(format_style_content(&pretty, &options).unwrap(), pretty);
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
