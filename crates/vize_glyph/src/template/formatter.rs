//! Core template formatter implementation.
//!
//! Contains the `TemplateFormatter` struct that drives the high-performance
//! template formatting pipeline, including tag parsing, attribute layout,
//! and interpolation formatting.

use crate::{error::FormatError, options::FormatOptions};
use memchr::memchr3;
use vize_l0::String;

use super::{
    attributes::sort_attributes,
    helpers::{
        byte_at, find_bytes, is_void_element_str, is_whitespace, parse_closing_tag, sub_slice,
    },
};

mod interpolation;
mod opening_attributes;
mod preserved_text;
mod suppression;
mod tags;
mod text;
mod whitespace_significant;

#[cfg(test)]
pub(crate) use interpolation::format_interpolations;
use interpolation::parse_interpolation_range;
use interpolation::{format_interpolation_expression, format_interpolations_with_vue_version};
use suppression::{LineJoiner, TextRun};
use whitespace_significant::is_whitespace_significant_element;

/// High-performance template formatter.
pub(crate) struct TemplateFormatter<'a> {
    options: &'a FormatOptions,
    vue_version: crate::VueVersion,
    indent: &'static [u8],
    newline: &'static [u8],
    base_depth: usize,
}

impl<'a> TemplateFormatter<'a> {
    #[inline]
    pub(crate) fn new(
        options: &'a FormatOptions,
        vue_version: crate::VueVersion,
        base_depth: usize,
    ) -> Self {
        Self {
            options,
            vue_version,
            indent: options.indent_bytes(),
            newline: options.newline_bytes(),
            base_depth,
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
            while pos < len && matches!(byte_at(source, pos), b' ' | b'\t' | b'\r') {
                pos += 1;
            }

            if pos >= len {
                break;
            }

            // Handle newlines
            if byte_at(source, pos) == b'\n' {
                pos += 1;
                continue;
            }

            if pos + 1 < len
                && byte_at(source, pos) == b'{'
                && byte_at(source, pos + 1) == b'{'
                && let Some((expr_start, expr_end, end_pos)) =
                    parse_interpolation_range(source, pos)
                && sub_slice(source, pos..end_pos).contains(&b'\n')
            {
                self.flush_text_buffer(&mut output, &mut text, depth, &mut joiner);
                let expr =
                    std::str::from_utf8(sub_slice(source, expr_start..expr_end)).unwrap_or("");
                let depth = joiner.interpolation_depth(pos, end_pos, depth);
                self.open_chunk(&mut output, depth, joiner.open(pos));
                self.write_multiline_interpolation(&mut output, expr, depth);
                joiner.finish(end_pos);
                pos = end_pos;
                continue;
            }

            // HTML comment <!-- ... -->
            if pos + 3 < len && sub_slice(source, pos..pos + 4) == b"<!--" {
                self.flush_text_buffer(&mut output, &mut text, depth, &mut joiner);
                let comment_start = pos;
                let join = joiner.open(comment_start);
                if let Some(end_offset) = find_bytes(sub_slice(source, pos..), b"-->") {
                    let comment_end = pos + end_offset + 3;
                    self.open_chunk(&mut output, depth, join);
                    output.extend_from_slice(sub_slice(source, comment_start..comment_end));
                    output.extend_from_slice(self.newline);
                    joiner.finish(comment_end);
                    pos = comment_end;
                } else {
                    // Unclosed comment - write remainder
                    self.open_chunk(&mut output, depth, join);
                    output.extend_from_slice(sub_slice(source, comment_start..));
                    output.extend_from_slice(self.newline);
                    joiner.finish(len);
                    pos = len;
                }
                continue;
            }

            // Tag start
            if byte_at(source, pos) == b'<' {
                if pos + 1 < len
                    && byte_at(source, pos + 1) == b'/'
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
                    self.parse_opening_tag(source, pos, depth)
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

                    let closing_bracket_on_own_line = self.write_opening_attributes_with_prefix(
                        &mut output,
                        &tag_name,
                        &sorted_attrs,
                        depth,
                        join.is_continuation() && !joiner.locks_current_line(),
                        if is_self_closing { 3 } else { 1 },
                    );

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
                            joiner.opened_element(pos, end_pos);
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
                let Some(offset) = memchr3(b'\n', b'<', b'{', sub_slice(source, pos..)) else {
                    pos = len;
                    break;
                };
                pos += offset;

                match byte_at(source, pos) {
                    b'\n' | b'<' => break,
                    b'{' if pos + 1 < len && byte_at(source, pos + 1) == b'{' => {
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
                while content_end > content_start && is_whitespace(byte_at(source, content_end - 1))
                {
                    content_end -= 1;
                }

                if content_end > content_start {
                    text.push_source(source, content_start, content_end);
                }
            }

            // Handle newline
            if pos < len && byte_at(source, pos) == b'\n' {
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

        // SAFETY: copied UTF-8 template ranges, formatter `&str` fragments and ASCII layout
        // keep valid bytes. Parser ranges and ASCII delimiter checks maintain UTF-8 boundaries.
        // Skipping validation preserves formatter throughput for large templates.
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
}
