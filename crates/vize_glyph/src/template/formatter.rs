//! Core template formatter implementation.
//!
//! Contains the `TemplateFormatter` struct that drives the high-performance
//! template formatting pipeline, including tag parsing, attribute layout,
//! and interpolation formatting.

use crate::{error::FormatError, options::FormatOptions, script};
use memchr::memchr3;
use vize_carton::{String, ToCompactString};

use super::{
    attributes::{ParsedAttribute, render_attribute, sort_attributes},
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

            if pos + 1 < len
                && source[pos] == b'{'
                && source[pos + 1] == b'{'
                && let Some((expr_start, expr_end, end_pos)) =
                    parse_interpolation_range(source, pos)
                && source[pos..end_pos].contains(&b'\n')
            {
                self.flush_text_buffer(&mut output, &mut line_buffer, depth);
                let expr = std::str::from_utf8(&source[expr_start..expr_end]).unwrap_or("");
                self.write_multiline_interpolation(&mut output, expr, depth);
                pos = end_pos;
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
                if pos + 1 < len
                    && source[pos + 1] == b'/'
                    && let Some((tag_name, end_pos)) = parse_closing_tag(source, pos)
                {
                    depth = depth.saturating_sub(1);
                    self.write_indent(&mut output, depth);
                    output.extend_from_slice(b"</");
                    output.extend_from_slice(tag_name.as_bytes());
                    output.push(b'>');
                    output.extend_from_slice(self.newline);
                    pos = end_pos;
                    continue;
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

                    let mut closing_bracket_on_own_line = false;
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
                                closing_bracket_on_own_line = true;
                            }
                        } else {
                            for attr in &sorted_attrs {
                                output.push(b' ');
                                output.extend_from_slice(render_attribute(attr).as_bytes());
                            }
                        }
                    }

                    if is_self_closing {
                        if closing_bracket_on_own_line {
                            output.extend_from_slice(b"/>");
                        } else {
                            output.extend_from_slice(b" />");
                        }
                    } else if !is_void_element_str(&tag_name)
                        && let Some(closing_end_pos) =
                            self.parse_immediate_empty_closing_tag(source, end_pos, &tag_name)
                    {
                        output.push(b'>');
                        output.extend_from_slice(b"</");
                        output.extend_from_slice(tag_name.as_bytes());
                        output.push(b'>');
                        output.extend_from_slice(self.newline);
                        pos = closing_end_pos;
                        continue;
                    } else if is_whitespace_significant_element(&tag_name, &sorted_attrs) {
                        // `<pre>`, `<textarea>`, and any element with `v-pre`
                        // are whitespace-significant. Their content must be
                        // emitted byte-for-byte: a formatter must never
                        // change rendered output. Find the matching close
                        // tag and copy the inner source verbatim. (#963)
                        output.push(b'>');
                        if let Some(close_start) =
                            find_matching_close_tag(source, end_pos, &tag_name)
                        {
                            output.extend_from_slice(&source[end_pos..close_start]);
                            output.extend_from_slice(b"</");
                            output.extend_from_slice(tag_name.as_bytes());
                            output.push(b'>');
                            output.extend_from_slice(self.newline);
                            // Move past `</tag_name>`
                            pos = close_start + 2 + tag_name.len() + 1;
                            continue;
                        } else {
                            // Unclosed — copy the rest and stop.
                            output.extend_from_slice(&source[end_pos..]);
                            pos = len;
                            continue;
                        }
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

        // SAFETY: `output` contains only copied ranges from the UTF-8 template
        // source, formatter-produced `&str` fragments, and ASCII indentation or
        // line breaks. The cursor moves across UTF-8 using the parser's byte
        // ranges and ASCII delimiter checks, so the buffer cannot contain an
        // invalid byte sequence. Skipping validation preserves formatter
        // throughput for large templates.
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

    fn write_multiline_interpolation(&self, output: &mut Vec<u8>, expr: &str, depth: usize) {
        self.write_indented_line(output, b"{{", depth);

        let formatted_expr = format_interpolation_expression(expr, self.options);
        for line in formatted_expr.trim().lines() {
            self.write_indent(output, depth + 1);
            output.extend_from_slice(line.trim_end_matches('\r').as_bytes());
            output.extend_from_slice(self.newline);
        }

        self.write_indented_line(output, b"}}", depth);
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

/// Returns true if the element's content must be preserved byte-for-byte:
/// `<pre>`, `<textarea>`, or any element with the `v-pre` directive.
/// Whitespace and interpolations inside these regions are rendered as-is
/// at runtime, so the formatter must not touch them. (#963)
fn is_whitespace_significant_element(tag_name: &str, attrs: &[ParsedAttribute]) -> bool {
    if matches!(
        tag_name,
        "pre" | "Pre" | "PRE" | "textarea" | "Textarea" | "TEXTAREA"
    ) {
        return true;
    }
    attrs
        .iter()
        .any(|attr| attr.name.eq_ignore_ascii_case("v-pre"))
}

/// Find the start of the matching `</tag_name>` for a content region that
/// begins at `start` in `source`. Returns the byte index of the `<` of the
/// closing tag, or `None` if no matching close is found.
///
/// This is a tag-name aware scan so nested elements with the same tag are
/// handled correctly (e.g. `<pre>...<pre>x</pre>...</pre>`).
fn find_matching_close_tag(source: &[u8], start: usize, tag_name: &str) -> Option<usize> {
    let len = source.len();
    let tag_bytes = tag_name.as_bytes();
    let mut pos = start;
    let mut depth: i32 = 1;
    while pos < len {
        let offset = memchr::memchr(b'<', &source[pos..])?;
        pos += offset;
        if pos + 1 >= len {
            return None;
        }
        // Skip comments and CDATA to avoid false matches inside them.
        if pos + 3 < len && &source[pos..pos + 4] == b"<!--" {
            if let Some(end) = find_bytes(&source[pos..], b"-->") {
                pos += end + 3;
                continue;
            }
            return None;
        }

        let is_closing = source[pos + 1] == b'/';
        let name_start = if is_closing { pos + 2 } else { pos + 1 };
        if name_start >= len {
            return None;
        }
        let name_bytes = &source[name_start..];
        if name_bytes.len() >= tag_bytes.len()
            && name_bytes[..tag_bytes.len()].eq_ignore_ascii_case(tag_bytes)
        {
            let after = name_bytes.get(tag_bytes.len()).copied().unwrap_or(0);
            let after_is_terminator = matches!(after, b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r');
            if after_is_terminator {
                if is_closing {
                    depth -= 1;
                    if depth == 0 {
                        return Some(pos);
                    }
                } else if !is_void_element_str(tag_name) {
                    // Treat self-closing forms (`<tag … />`) as not opening
                    // a new nesting level. Peek to the next `>` and check
                    // for a preceding `/`.
                    if let Some(gt) = memchr::memchr(b'>', &source[pos..]) {
                        let close_at = pos + gt;
                        if close_at > 0 && source[close_at - 1] != b'/' {
                            depth += 1;
                        }
                        pos = close_at + 1;
                        continue;
                    }
                }
            }
        }
        pos += 1;
    }
    None
}
