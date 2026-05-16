//! Core template formatter implementation.
//!
//! Contains the `TemplateFormatter` struct that drives the high-performance
//! template formatting pipeline, including tag parsing, attribute layout,
//! and interpolation formatting.

use crate::{error::FormatError, options::FormatOptions, script};
use vize_carton::{String, ToCompactString};

use super::{
    attributes::{render_attribute, sort_attributes, ParsedAttribute},
    directives::normalize_attribute,
    helpers::{
        find_bytes, is_tag_name_char, is_void_element_str, is_whitespace, parse_closing_tag,
    },
};

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
        let mut line_buffer = Vec::with_capacity(256);

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

            // HTML comment <!-- ... -->
            if pos + 3 < len && &source[pos..pos + 4] == b"<!--" {
                self.flush_text_buffer(&mut output, &mut line_buffer, depth);
                let comment_start = pos;
                if let Some(end_offset) = find_bytes(&source[pos..], b"-->") {
                    let comment_end = pos + end_offset + 3;
                    self.write_indent(&mut output, depth);
                    output.extend_from_slice(&source[comment_start..comment_end]);
                    output.extend_from_slice(self.newline);
                    pos = comment_end;
                } else {
                    // Unclosed comment - write remainder
                    self.write_indent(&mut output, depth);
                    output.extend_from_slice(&source[comment_start..]);
                    output.extend_from_slice(self.newline);
                    pos = len;
                }
                continue;
            }

            // Tag start
            if source[pos] == b'<' {
                self.flush_text_buffer(&mut output, &mut line_buffer, depth);

                // Closing tag
                if pos + 1 < len && source[pos + 1] == b'/' {
                    if let Some((tag_name, end_pos)) = parse_closing_tag(source, pos) {
                        depth = depth.saturating_sub(1);
                        self.write_indent(&mut output, depth);
                        output.extend_from_slice(b"</");
                        output.extend_from_slice(tag_name.as_bytes());
                        output.push(b'>');
                        output.extend_from_slice(self.newline);
                        pos = end_pos;
                        continue;
                    }
                }

                // Opening tag
                if let Some((tag_name, attrs, is_self_closing, end_pos)) =
                    self.parse_opening_tag(source, pos)
                {
                    // Sort attributes if enabled
                    let mut sorted_attrs = attrs;
                    if self.options.sort_attributes {
                        sort_attributes(&mut sorted_attrs, self.options);
                    }

                    self.write_indent(&mut output, depth);
                    output.push(b'<');
                    output.extend_from_slice(tag_name.as_bytes());

                    if !sorted_attrs.is_empty() {
                        let use_multiline =
                            self.should_use_multiline_attrs(&tag_name, &sorted_attrs, depth);

                        if use_multiline {
                            let max_per_line = self
                                .options
                                .max_attributes_per_line
                                .unwrap_or(1) // default 1 when multiline
                                .max(1) as usize;

                            let mut line_count = 0;
                            for attr in &sorted_attrs {
                                if line_count == 0 {
                                    // Start a new attribute line
                                    output.extend_from_slice(self.newline);
                                    self.write_indent(&mut output, depth + 1);
                                } else {
                                    output.push(b' ');
                                }
                                output.extend_from_slice(render_attribute(attr).as_bytes());
                                line_count += 1;
                                if line_count >= max_per_line {
                                    line_count = 0;
                                }
                            }
                            if !self.options.bracket_same_line {
                                output.extend_from_slice(self.newline);
                                self.write_indent(&mut output, depth);
                            }
                        } else {
                            for attr in &sorted_attrs {
                                output.push(b' ');
                                output.extend_from_slice(render_attribute(attr).as_bytes());
                            }
                        }
                    }

                    if is_self_closing {
                        output.extend_from_slice(b" />");
                    } else {
                        output.push(b'>');
                        if !is_void_element_str(&tag_name) {
                            depth += 1;
                        }
                    }
                    output.extend_from_slice(self.newline);
                    pos = end_pos;
                    continue;
                }
            }

            // Accumulate text content until newline or tag
            let content_start = pos;
            while pos < len && source[pos] != b'\n' && source[pos] != b'<' {
                pos += 1;
            }

            if pos > content_start {
                // Trim trailing whitespace from content
                let mut content_end = pos;
                while content_end > content_start && is_whitespace(source[content_end - 1]) {
                    content_end -= 1;
                }

                if content_end > content_start {
                    if !line_buffer.is_empty() {
                        line_buffer.push(b' ');
                    }
                    line_buffer.extend_from_slice(&source[content_start..content_end]);
                }
            }

            // Handle newline
            if pos < len && source[pos] == b'\n' {
                self.flush_text_buffer(&mut output, &mut line_buffer, depth);
                pos += 1;
            }
        }

        // Flush remaining content
        self.flush_text_buffer(&mut output, &mut line_buffer, depth);

        // Remove trailing newline for consistency
        while output.last().is_some_and(|&b| b == b'\n' || b == b'\r') {
            output.pop();
        }

        // SAFETY: We only wrote valid UTF-8 bytes
        Ok(unsafe { String::from_utf8_unchecked(output) })
    }

    /// Flush accumulated text content with interpolation formatting.
    #[inline]
    fn flush_text_buffer(&self, output: &mut Vec<u8>, buffer: &mut Vec<u8>, depth: usize) {
        if buffer.is_empty() {
            return;
        }
        let text = std::str::from_utf8(buffer).unwrap_or("");
        let formatted = format_interpolations(text, self.options);
        self.write_indented_line(output, formatted.as_bytes(), depth);
        buffer.clear();
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

    /// Determine whether attributes should be rendered in multiline mode.
    fn should_use_multiline_attrs(
        &self,
        tag_name: &str,
        attrs: &[ParsedAttribute],
        depth: usize,
    ) -> bool {
        if attrs.len() <= 1 {
            return false;
        }

        // Explicit max_attributes_per_line takes priority
        if let Some(max) = self.options.max_attributes_per_line {
            return attrs.len() > max as usize;
        }

        // single_attribute_per_line
        if self.options.single_attribute_per_line {
            return true;
        }

        // Check if all attributes on one line would exceed print_width
        let indent_len = self.indent.len() * depth;
        let tag_len = 1 + tag_name.len(); // '<' + tag_name
        let attrs_len: usize = attrs
            .iter()
            .map(|a| 1 + render_attribute(a).len()) // ' ' + attr
            .sum();
        let closing_len = 1; // '>'
        let total = indent_len + tag_len + attrs_len + closing_len;

        total > self.options.print_width as usize
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
        let (name, value, priority) = normalize_attribute(&raw_name, value, self.options);

        (
            Some(ParsedAttribute {
                name,
                value,
                priority,
                original_index: index,
            }),
            pos,
        )
    }
}

/// Format interpolations in text content: `{{expr}}` -> `{{ expr }}`.
pub(crate) fn format_interpolations(text: &str, options: &FormatOptions) -> String {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + 16);
    let mut pos = 0;

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
                let formatted_expr = script::format_js_expression(expr, options)
                    .unwrap_or_else(|| expr.trim().to_compact_string());
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
            // Push one UTF-8 character
            if let Some(ch) = text[pos..].chars().next() {
                result.push(ch);
                pos += ch.len_utf8();
            } else {
                pos += 1;
            }
        }
    }

    result
}
