//! CSS formatting using lightningcss.
//!
//! This module provides formatting for CSS/SCSS/Less content
//! in Vue SFC `<style>` blocks using lightningcss for parsing and printing.

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
                    separator_lines = separator_lines.max(source_separator_lines(segment.content));
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
                    for _ in 0..separator_lines.max(source_separator_lines_before(segment.content))
                    {
                        output.push_str(newline);
                    }
                }
                output.push_str(formatted);
                emitted_any = true;
                separator_lines = source_separator_lines_after(segment.content);
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
    if changes_authored_css(trimmed, formatted.as_str()) {
        Ok(format_layout_only(trimmed, options))
    } else {
        Ok(formatted)
    }
}

/// Indent rules and declarations without passing authored tokens through the
/// CSS printer. This path keeps media queries, selectors, and values intact.
fn format_layout_only(source: &str, options: &FormatOptions) -> String {
    let newline = options.newline_string();
    let indent = options.indent_string();
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len() + source.len() / 4);
    let mut depth = 0usize;
    let mut parens = 0usize;
    let mut brackets = 0usize;
    let mut quote = None;
    let mut start = 0usize;
    let mut index = 0usize;

    while let Some(&byte) = bytes.get(index) {
        if byte == b'\\' && index + 1 < bytes.len() {
            index += 2;
            continue;
        }
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
            index += 1;
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            index += 2;
            while index + 1 < bytes.len() && bytes.get(index..index + 2) != Some(b"*/".as_slice()) {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
            continue;
        }
        match byte {
            b'(' => parens += 1,
            b')' => parens = parens.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            b'{' if parens == 0 && brackets == 0 => {
                write_css_line(
                    &mut output,
                    source.get(start..index).unwrap_or_default().trim(),
                    depth,
                    &indent,
                    newline,
                );
                // Replace the preceding newline with the opening brace.
                if output.ends_with(newline) {
                    output.truncate(output.len() - newline.len());
                }
                output.push_str(" {");
                output.push_str(newline);
                depth += 1;
                start = index + 1;
            }
            b';' if parens == 0 && brackets == 0 => {
                let statement = source.get(start..index).unwrap_or_default().trim();
                if !statement.is_empty() {
                    write_css_indent(&mut output, depth, &indent);
                    output.push_str(statement);
                    output.push(';');
                    output.push_str(newline);
                }
                start = index + 1;
            }
            b'}' if parens == 0 && brackets == 0 && depth > 0 => {
                write_css_line(
                    &mut output,
                    source.get(start..index).unwrap_or_default().trim(),
                    depth,
                    &indent,
                    newline,
                );
                depth -= 1;
                write_css_indent(&mut output, depth, &indent);
                output.push('}');
                output.push_str(newline);
                if depth == 0
                    && !source
                        .get(index + 1..)
                        .unwrap_or_default()
                        .trim()
                        .is_empty()
                {
                    output.push_str(newline);
                }
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    write_css_line(
        &mut output,
        source.get(start..).unwrap_or_default().trim(),
        depth,
        &indent,
        newline,
    );
    output
}

fn write_css_line(output: &mut String, content: &str, depth: usize, indent: &str, newline: &str) {
    if !content.is_empty() {
        write_css_indent(output, depth, indent);
        output.push_str(content);
        output.push_str(newline);
    }
}

fn write_css_indent(output: &mut String, depth: usize, indent: &str) {
    for _ in 0..depth {
        output.push_str(indent);
    }
}

fn source_separator_lines(source: &str) -> usize {
    if source
        .as_bytes()
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count()
        >= 2
    {
        2
    } else {
        1
    }
}

fn source_separator_lines_before(source: &str) -> usize {
    let prefix_len = source.len() - source.trim_start().len();
    source_separator_lines(source.get(..prefix_len).unwrap_or_default())
}

fn source_separator_lines_after(source: &str) -> usize {
    source_separator_lines(source.get(source.trim_end().len()..).unwrap_or_default())
}

fn changes_authored_css(source: &str, printed: &str) -> bool {
    // Ignore layout whitespace and an optional final declaration semicolon.
    // All other token changes, including inserted nesting ampersands, changed
    // media features, reordered values and shortened pseudo-elements, matter.
    fn tokens(source: &str) -> Vec<u8> {
        let bytes = source.as_bytes();
        let mut output = Vec::with_capacity(bytes.len());
        let mut index = 0;
        let mut quote = None;
        while let Some(&byte) = bytes.get(index) {
            if byte == b'\\' && index + 1 < bytes.len() {
                output.extend_from_slice(bytes.get(index..index + 2).unwrap_or_default());
                index += 2;
                continue;
            }
            if let Some(delimiter) = quote {
                output.push(byte);
                if byte == delimiter {
                    quote = None;
                }
                index += 1;
                continue;
            }
            if matches!(byte, b'\'' | b'"') {
                quote = Some(byte);
                output.push(byte);
                index += 1;
                continue;
            }
            if byte.is_ascii_whitespace() {
                index += 1;
                continue;
            }
            if byte == b';'
                && bytes
                    .get(index + 1..)
                    .unwrap_or_default()
                    .iter()
                    .find(|b| !b.is_ascii_whitespace())
                    == Some(&b'}')
            {
                index += 1;
                continue;
            }
            output.push(byte);
            index += 1;
        }
        output
    }
    tokens(source) != tokens(printed)
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
