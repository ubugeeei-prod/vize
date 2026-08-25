//! Core template formatter implementation.
//!
//! Contains the `TemplateFormatter` struct that drives the high-performance
//! template formatting pipeline, including tag parsing, attribute layout,
//! and interpolation formatting.

use crate::{error::FormatError, options::FormatOptions, script};
use memchr::memchr3;
use vize_s0::{String, ToCompactString};

use super::{
    attributes::{
        ParsedAttribute, render_attribute, should_use_multiline_attrs, sort_attributes,
        write_rendered_attributes,
    },
    directives::normalize_attribute,
    helpers::{
        find_bytes, is_tag_name_char, is_void_element_str, is_whitespace, parse_closing_tag,
    },
};

mod interpolation;
mod suppression;
mod text;
mod whitespace_significant;

use suppression::{LineJoiner, TextRun};
use whitespace_significant::is_whitespace_significant_element;

/// High-performance template formatter.
pub(crate) struct TemplateFormatter<'a> {
    options: &'a FormatOptions,
    indent: &'static [u8],
    newline: &'static [u8],
}

impl<'a> TemplateFormatter<'a> {
    #[inline]
    pub(crate) fn new(options: &'a FormatOptions) -> Self {
        Self {
            options,
            indent: options.indent_bytes(),
            newline: options.newline_bytes(),
        }
    }

    pub(crate) fn format(&self, source: &[u8]) -> Result<String, FormatError> {
        let len = source.len();
        let mut output = Vec::with_capacity(len + len / 4);
        let mut pos = 0;
        let mut depth: usize = 0;
        let mut text = TextRun::new();
        // Lines a line-scoped lint suppression covers must not be split. (#3343)
        let mut joiner = LineJoiner::new(source);

        while pos < len {
            // Skip whitespace at line start (except newlines)
            while pos < len && is_whitespace(source[pos]) && source[pos] != b'\n' {
                pos += 1;
            }

            if pos >= len {
                break;
            }

            // Handle newlines
            if source[pos] == b'\n' {
                pos += 1;
                continue;
            }

            if pos + 1 < len
                && source[pos] == b'{'
                && source[pos + 1] == b'{'
                && let Some((expr_start, expr_end, end_pos)) =
                    parse_interpolation_range(source, pos)
                && source[pos..end_pos].contains(&b'\n')
            {
                self.flush_text_buffer(&mut output, &mut text, depth, &mut joiner);
                let expr = std::str::from_utf8(&source[expr_start..expr_end]).unwrap_or("");
                self.open_chunk(&mut output, depth, joiner.open(pos));
                self.write_multiline_interpolation(&mut output, expr, depth);
                joiner.finish(end_pos);
                pos = end_pos;
                continue;
            }

            // HTML comment <!-- ... -->
            if pos + 3 < len && &source[pos..pos + 4] == b"<!--" {
                self.flush_text_buffer(&mut output, &mut text, depth, &mut joiner);
                let comment_start = pos;
                let join = joiner.open(comment_start);
                if let Some(end_offset) = find_bytes(&source[pos..], b"-->") {
                    let comment_end = pos + end_offset + 3;
                    self.open_chunk(&mut output, depth, join);
                    output.extend_from_slice(&source[comment_start..comment_end]);
                    output.extend_from_slice(self.newline);
                    joiner.finish(comment_end);
                    pos = comment_end;
                } else {
                    // Unclosed comment - write remainder
                    self.open_chunk(&mut output, depth, join);
                    output.extend_from_slice(&source[comment_start..]);
                    output.extend_from_slice(self.newline);
                    joiner.finish(len);
                    pos = len;
                }
                continue;
            }

            // Tag start
            if source[pos] == b'<' {
                if pos + 1 < len
                    && source[pos + 1] == b'/'
                    && let Some((tag_name, end_pos)) = parse_closing_tag(source, pos)
                {
                    self.flush_text_buffer(&mut output, &mut text, depth, &mut joiner);
                    let join = joiner.open(pos);
                    depth = depth.saturating_sub(1);
                    self.open_chunk(&mut output, depth, join);
                    output.extend_from_slice(b"</");
                    output.extend_from_slice(tag_name.as_bytes());
                    output.push(b'>');
                    output.extend_from_slice(self.newline);
                    joiner.finish(end_pos);
                    pos = end_pos;
                    continue;
                }
                if let Some((tag_name, attrs, is_self_closing, end_pos)) =
                    self.parse_opening_tag(source, pos)
                {
                    self.flush_text_buffer(&mut output, &mut text, depth, &mut joiner);
                    let join = joiner.open(pos);
                    let mut sorted_attrs = attrs;
                    if self.options.sort_attributes {
                        sort_attributes(&mut sorted_attrs, self.options);
                    }

                    self.open_chunk(&mut output, depth, join);
                    output.push(b'<');
                    output.extend_from_slice(tag_name.as_bytes());

                    let mut closing_bracket_on_own_line = false;
                    if !sorted_attrs.is_empty() {
                        // Render each attribute exactly once; both the
                        // multiline decision and emission below reuse this.
                        let mut rendered: Vec<String> = Vec::with_capacity(sorted_attrs.len());
                        rendered.extend(sorted_attrs.iter().map(render_attribute));

                        let use_multiline = should_use_multiline_attrs(
                            self.options,
                            &tag_name,
                            &sorted_attrs,
                            &rendered,
                            depth,
                            self.indent,
                        );

                        if use_multiline {
                            let max_per_line = if self.options.single_attribute_per_line {
                                1
                            } else {
                                self.options
                                    .max_attributes_per_line
                                    .unwrap_or(1) // default 1 when multiline
                                    .max(1) as usize
                            };

                            write_rendered_attributes(
                                &mut output,
                                &sorted_attrs,
                                &rendered,
                                self.newline,
                                self.indent,
                                depth + 1,
                                max_per_line,
                            );
                            if !self.options.bracket_same_line {
                                output.extend_from_slice(self.newline);
                                self.write_indent(&mut output, depth);
                                closing_bracket_on_own_line = true;
                            }
                        } else {
                            for attr in &rendered {
                                output.push(b' ');
                                output.extend_from_slice(attr.as_bytes());
                            }
                        }
                    }

                    // Compute once per opening tag; consumed in the two
                    // void-element branches below.
                    let is_void = is_void_element_str(&tag_name);
                    if is_self_closing {
                        if closing_bracket_on_own_line {
                            output.extend_from_slice(b"/>");
                        } else {
                            output.extend_from_slice(b" />");
                        }
                    } else if !is_void
                        && !is_whitespace_significant_element(&tag_name, &sorted_attrs)
                        && let Some(closing_end_pos) =
                            self.parse_immediate_empty_closing_tag(source, end_pos, &tag_name)
                    {
                        output.push(b'>');
                        output.extend_from_slice(b"</");
                        output.extend_from_slice(tag_name.as_bytes());
                        output.push(b'>');
                        output.extend_from_slice(self.newline);
                        joiner.finish(closing_end_pos);
                        pos = closing_end_pos;
                        continue;
                    } else if is_whitespace_significant_element(&tag_name, &sorted_attrs) {
                        // Copy `<pre>`/`<textarea>`/`v-pre` content verbatim so
                        // the formatter never changes rendered output.
                        // (#963, #3249)
                        pos = self.copy_whitespace_significant_element(
                            source,
                            end_pos,
                            &tag_name,
                            len,
                            &mut output,
                        );
                        joiner.finish(pos);
                        continue;
                    } else {
                        output.push(b'>');
                        if !is_void {
                            depth += 1;
                        }
                    }
                    output.extend_from_slice(self.newline);
                    joiner.finish(end_pos);
                    pos = end_pos;
                    continue;
                }
                // Keep a non-tag `<` as text and advance past it.
                text.push_byte(pos, b'<');
                pos += 1;
                continue;
            }

            // Accumulate text content until newline or tag
            let content_start = pos;
            while pos < len {
                let Some(offset) = memchr3(b'\n', b'<', b'{', &source[pos..]) else {
                    pos = len;
                    break;
                };
                pos += offset;

                match source[pos] {
                    b'\n' | b'<' => break,
                    b'{' if pos + 1 < len && source[pos + 1] == b'{' => {
                        if let Some((_, _, end_pos)) = parse_interpolation_range(source, pos) {
                            pos = end_pos;
                        } else {
                            pos += 1;
                        }
                    }
                    _ => pos += 1,
                }
            }

            if pos > content_start {
                // Trim trailing whitespace from content
                let mut content_end = pos;
                while content_end > content_start && is_whitespace(source[content_end - 1]) {
                    content_end -= 1;
                }

                if content_end > content_start {
                    text.push_source(source, content_start, content_end);
                }
            }

            // Handle newline
            if pos < len && source[pos] == b'\n' {
                self.flush_text_buffer(&mut output, &mut text, depth, &mut joiner);
                pos += 1;
            }
        }

        // Flush remaining content
        self.flush_text_buffer(&mut output, &mut text, depth, &mut joiner);

        // Remove trailing newline for consistency
        while output.last().is_some_and(|&b| b == b'\n' || b == b'\r') {
            output.pop();
        }

        // SAFETY: `output` contains only copied ranges from the UTF-8 template
        // source, formatter-produced `&str` fragments, and ASCII indentation or
        // line breaks. The cursor moves across UTF-8 using the parser's byte
        // ranges and ASCII delimiter checks, so the buffer cannot contain an
        // invalid byte sequence. Skipping validation preserves formatter
        // throughput for large templates.
        Ok(unsafe { String::from_utf8_unchecked(output) })
    }

    #[inline]
    fn write_indent(&self, output: &mut Vec<u8>, depth: usize) {
        for _ in 0..depth {
            output.extend_from_slice(self.indent);
        }
    }

    #[inline]
    fn write_indented_line(&self, output: &mut Vec<u8>, content: &[u8], depth: usize) {
        self.write_indent(output, depth);
        output.extend_from_slice(content);
        output.extend_from_slice(self.newline);
    }

    /// Parse an opening tag into structured attributes.
    fn parse_opening_tag(
        &self,
        source: &[u8],
        start: usize,
    ) -> Option<(String, Vec<ParsedAttribute>, bool, usize)> {
        let len = source.len();
        let mut pos = start + 1; // Skip '<'

        // Parse tag name
        let tag_start = pos;
        while pos < len && is_tag_name_char(source[pos]) {
            pos += 1;
        }
        if pos == tag_start {
            return None;
        }

        let tag_name = std::str::from_utf8(&source[tag_start..pos])
            .unwrap_or("")
            .to_compact_string();

        // Parse attributes
        let mut attrs = Vec::new();
        let mut is_self_closing = false;
        let mut attr_index: usize = 0;

        while pos < len && source[pos] != b'>' {
            // Skip whitespace
            while pos < len && is_whitespace(source[pos]) {
                pos += 1;
            }
            if pos >= len {
                break;
            }

            // Check for self-closing or end
            if source[pos] == b'/' {
                is_self_closing = true;
                pos += 1;
                continue;
            }
            if source[pos] == b'>' {
                break;
            }

            // Parse single attribute
            let (attr, new_pos) = self.parse_single_attribute(source, pos, attr_index);
            if let Some(attr) = attr {
                attrs.push(attr);
                attr_index += 1;
            }
            pos = new_pos;
        }

        // Skip '>'
        if pos < len && source[pos] == b'>' {
            pos += 1;
        }

        Some((tag_name, attrs, is_self_closing, pos))
    }

    /// Return the end of an immediately following matching closing tag.
    fn parse_immediate_empty_closing_tag(
        &self,
        source: &[u8],
        start: usize,
        tag_name: &str,
    ) -> Option<usize> {
        let len = source.len();
        let mut pos = start;

        while pos < len && is_whitespace(source[pos]) {
            pos += 1;
        }

        if pos + 1 >= len || source[pos] != b'<' || source[pos + 1] != b'/' {
            return None;
        }

        let (closing_tag_name, end_pos) = parse_closing_tag(source, pos)?;
        if closing_tag_name.as_str() == tag_name {
            Some(end_pos)
        } else {
            None
        }
    }

    /// Parse a single attribute: name, optional `="value"`.
    fn parse_single_attribute(
        &self,
        source: &[u8],
        start: usize,
        index: usize,
    ) -> (Option<ParsedAttribute>, usize) {
        let len = source.len();
        let mut pos = start;

        // Parse attribute name (may include :, @, #, ., v-, etc.)
        let name_start = pos;
        while pos < len {
            let b = source[pos];
            if is_whitespace(b) || b == b'>' || b == b'/' || b == b'=' {
                break;
            }
            pos += 1;
        }

        if pos == name_start {
            // Skip unknown byte to avoid infinite loop
            return (None, pos + 1);
        }

        let raw_name = std::str::from_utf8(&source[name_start..pos])
            .unwrap_or("")
            .to_compact_string();

        // Skip whitespace before '='
        let mut val_pos = pos;
        while val_pos < len && (source[val_pos] == b' ' || source[val_pos] == b'\t') {
            val_pos += 1;
        }

        // Check for '=' and value
        let value = if val_pos < len && source[val_pos] == b'=' {
            val_pos += 1; // skip '='

            // Skip whitespace after '='
            while val_pos < len && (source[val_pos] == b' ' || source[val_pos] == b'\t') {
                val_pos += 1;
            }

            if val_pos < len && (source[val_pos] == b'"' || source[val_pos] == b'\'') {
                // Quoted value
                let quote = source[val_pos];
                val_pos += 1;
                let value_start = val_pos;
                while val_pos < len && source[val_pos] != quote {
                    val_pos += 1;
                }
                let value = std::str::from_utf8(&source[value_start..val_pos])
                    .unwrap_or("")
                    .to_compact_string();
                if val_pos < len {
                    val_pos += 1; // skip closing quote
                }
                pos = val_pos;
                Some(value)
            } else {
                // Unquoted value
                let value_start = val_pos;
                while val_pos < len
                    && !is_whitespace(source[val_pos])
                    && source[val_pos] != b'>'
                    && source[val_pos] != b'/'
                {
                    val_pos += 1;
                }
                let value = std::str::from_utf8(&source[value_start..val_pos])
                    .unwrap_or("")
                    .to_compact_string();
                pos = val_pos;
                Some(value)
            }
        } else {
            // Boolean attribute (no value)
            None
        };

        // Normalize directives and determine priority
        let (name, value, priority, indent_multiline_value) =
            normalize_attribute(&raw_name, value, self.options);

        (
            Some(ParsedAttribute {
                name,
                value,
                priority,
                original_index: index,
                indent_multiline_value,
            }),
            pos,
        )
    }
}

/// Format interpolations in text content: `{{expr}}` -> `{{ expr }}`.
pub(crate) fn format_interpolations(text: &str, options: &FormatOptions) -> String {
    let bytes = text.as_bytes();
    let len = bytes.len();

    // Fast path: no `{` at all means no interpolations and no special bytes,
    // so the text is returned verbatim with a single allocation.
    let Some(first_brace) = memchr::memchr(b'{', bytes) else {
        return text.to_compact_string();
    };

    let mut result = String::with_capacity(len + 16);
    // Everything before the first `{` is ordinary text; copy it in one shot.
    result.push_str(&text[..first_brace]);
    let mut pos = first_brace;

    while pos < len {
        if pos + 1 < len && bytes[pos] == b'{' && bytes[pos + 1] == b'{' {
            // Find closing }}
            let expr_start = pos + 2;
            let mut depth = 1;
            let mut expr_end = expr_start;

            while expr_end + 1 < len {
                if bytes[expr_end] == b'{' && bytes[expr_end + 1] == b'{' {
                    depth += 1;
                    expr_end += 2;
                } else if bytes[expr_end] == b'}' && bytes[expr_end + 1] == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    expr_end += 2;
                } else {
                    expr_end += 1;
                }
            }

            if depth == 0 {
                let expr = &text[expr_start..expr_end];
                let formatted_expr = format_interpolation_expression(expr, options);
                result.push_str("{{ ");
                result.push_str(&formatted_expr);
                result.push_str(" }}");
                pos = expr_end + 2;
            } else {
                // Unclosed interpolation -- keep as-is
                result.push('{');
                pos += 1;
            }
        } else {
            // Ordinary text. Copy the run up to (but not including) the next
            // `{` in a single push instead of char-by-char. A lone `{` (one
            // not starting a `{{`) is emitted and stepped over individually,
            // exactly as before.
            let rest = &bytes[pos + 1..];
            let next = memchr::memchr(b'{', rest).map_or(len, |off| pos + 1 + off);
            result.push_str(&text[pos..next]);
            pos = next;
        }
    }

    result
}

fn format_interpolation_expression(expr: &str, options: &FormatOptions) -> String {
    script::format_js_expression(expr, options).unwrap_or_else(|| expr.trim().to_compact_string())
}

fn parse_interpolation_range(source: &[u8], start: usize) -> Option<(usize, usize, usize)> {
    let len = source.len();
    if start + 1 >= len || source[start] != b'{' || source[start + 1] != b'{' {
        return None;
    }

    let expr_start = start + 2;
    let mut depth = 1;
    let mut pos = expr_start;

    while pos + 1 < len {
        if source[pos] == b'{' && source[pos + 1] == b'{' {
            depth += 1;
            pos += 2;
        } else if source[pos] == b'}' && source[pos + 1] == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some((expr_start, pos, pos + 2));
            }
            pos += 2;
        } else {
            pos += 1;
        }
    }

    None
}
