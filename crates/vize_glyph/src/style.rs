//! CSS formatting using lightningcss.
//!
//! This module provides formatting for CSS/SCSS/Less content
//! in Vue SFC `<style>` blocks using lightningcss for parsing and printing.

use crate::error::FormatError;
use crate::options::FormatOptions;
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use vize_carton::{String, ToCompactString};

/// Format CSS content using lightningcss.
///
/// Top-level (depth 0) comments are extracted before parsing and re-inserted
/// at their original boundaries, because lightningcss drops non-license
/// comments during parse. Comments nested inside a selector block are still
/// dropped — that limitation is documented and covered by an ignored test.
pub fn format_style_content(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(String::default());
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

#[derive(Clone, Copy)]
enum SegmentKind {
    Code,
    Comment,
}

struct CssSegment<'a> {
    kind: SegmentKind,
    content: &'a str,
}

/// Split CSS source into alternating code and top-level comment segments.
///
/// Only depth-0 comments (those outside `{ ... }` and not inside a string
/// literal) become separate `Comment` segments. Comments nested in a selector
/// block stay embedded in the surrounding `Code` segment, because slicing the
/// rule at that point would produce text lightningcss could not parse.
fn split_top_level_comments(source: &str) -> Vec<CssSegment<'_>> {
    let bytes = source.as_bytes();
    let mut segments: Vec<CssSegment<'_>> = Vec::new();
    let mut depth: u32 = 0;
    let mut in_string: Option<u8> = None;
    let mut last_split = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        let c = bytes[i];

        if let Some(quote) = in_string {
            if c == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if c == quote {
                in_string = None;
            }
            i += 1;
            continue;
        }

        match c {
            b'"' | b'\'' => {
                in_string = Some(c);
                i += 1;
            }
            b'{' => {
                depth = depth.saturating_add(1);
                i += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                let comment_end = find_comment_end(bytes, i + 2);
                if depth == 0 {
                    if i > last_split {
                        segments.push(CssSegment {
                            kind: SegmentKind::Code,
                            content: &source[last_split..i],
                        });
                    }
                    segments.push(CssSegment {
                        kind: SegmentKind::Comment,
                        content: &source[i..comment_end],
                    });
                    last_split = comment_end;
                }
                i = comment_end;
            }
            _ => i += 1,
        }
    }

    if last_split < bytes.len() {
        segments.push(CssSegment {
            kind: SegmentKind::Code,
            content: &source[last_split..],
        });
    }

    segments
}

fn find_comment_end(bytes: &[u8], from: usize) -> usize {
    let mut j = from;
    while j + 1 < bytes.len() {
        if bytes[j] == b'*' && bytes[j + 1] == b'/' {
            return j + 2;
        }
        j += 1;
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::{FormatOptions, format_style_content};

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
    fn test_format_keeps_comment_like_content_inside_strings_as_string() {
        let source = ".x { content: \"/* not a comment */\"; }";
        let options = FormatOptions::default();
        let result = format_style_content(source, &options).unwrap();
        // The content stays as string literal, no segment split happens, so
        // lightningcss emits a clean single-rule output.
        assert!(result.contains(".x"));
        assert!(result.contains("\"/* not a comment */\""));
    }
}
