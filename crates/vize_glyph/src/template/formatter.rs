//! Core template formatter implementation.
//!
//! Contains the `TemplateFormatter` struct that drives the high-performance
//! template formatting pipeline, including tag parsing, attribute layout,
//! and interpolation formatting.

use crate::{error::FormatError, options::FormatOptions};
use memchr::memchr3;
use unicode_width::UnicodeWidthStr;
use vize_l0::String;
use vize_l0::string_builder::TextBuf;

use super::{
    attributes::sort_attributes,
    helpers::{
        byte_at, find_bytes, is_void_element_str, is_whitespace, parse_closing_tag, sub_slice,
    },
};

mod child_width;
mod closing_width;
mod inline_breaks;
mod interpolation;
pub(crate) mod nesting;
mod opening_attributes;
use opening_attributes::Continued;
mod preserved_text;
mod suppression;
mod tags;
mod text;
mod text_wrap;
mod whitespace_significant;

use inline_breaks::InlineNode;
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
    indent: &'static str,
    newline: &'static str,
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
            indent: options.indent_str(),
            newline: options.newline_string(),
            base_depth,
        }
    }

    pub(crate) fn format(&self, input: &str) -> Result<String, FormatError> {
        // Select the CR-aware scanner once per template. The const parameter
        // specializes the ordinary LF path back to one delimiter scan per text
        // run, while leaving raw-region bytes untouched.
        if self.newline == "\r" && memchr::memchr(b'\n', input.as_bytes()).is_none() {
            if self.options.canonical_directive_layout {
                self.format_inner::<true, true>(input)
            } else {
                self.format_inner::<true, false>(input)
            }
        } else if self.options.canonical_directive_layout {
            self.format_inner::<false, true>(input)
        } else {
            self.format_inner::<false, false>(input)
        }
    }

    fn format_inner<const HAS_CR: bool, const CANONICAL: bool>(
        &self,
        input: &str,
    ) -> Result<String, FormatError> {
        let source = input.as_bytes();
        let len = source.len();
        let mut output = TextBuf::with_capacity(len + len / 4);
        let mut pos = 0;
        let mut depth: usize = 0;
        let mut text = TextRun::new();
        let mut inline = InlineNode::None;
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
                && if HAS_CR {
                    memchr::memchr2(b'\n', b'\r', sub_slice(source, pos..end_pos)).is_some()
                } else {
                    memchr::memchr(b'\n', sub_slice(source, pos..end_pos)).is_some()
                }
            {
                self.flush_text_buffer::<CANONICAL>(&mut output, &mut text, depth, &mut joiner);
                let expr = input.get(expr_start..expr_end).unwrap_or_default();
                inline = InlineNode::None;
                let depth = joiner.interpolation_depth(pos, end_pos, depth);
                self.open_chunk(&mut output, depth, joiner.open(pos));
                self.write_multiline_interpolation(&mut output, expr, depth);
                joiner.finish(end_pos);
                pos = end_pos;
                continue;
            }

            // HTML comment <!-- ... -->
            if pos + 3 < len && sub_slice(source, pos..pos + 4) == b"<!--" {
                self.flush_text_buffer::<CANONICAL>(&mut output, &mut text, depth, &mut joiner);
                let comment_start = pos;
                inline = InlineNode::None;
                let join = joiner.open(comment_start);
                if let Some(end_offset) = find_bytes(sub_slice(source, pos..), b"-->") {
                    let comment_end = pos + end_offset + 3;
                    self.open_chunk(&mut output, depth, join);
                    output.push_str(input.get(comment_start..comment_end).unwrap_or_default());
                    output.push_str(self.newline);
                    joiner.finish(comment_end);
                    pos = comment_end;
                } else {
                    // Unclosed comment - write remainder
                    self.open_chunk(&mut output, depth, join);
                    output.push_str(input.get(comment_start..).unwrap_or_default());
                    output.push_str(self.newline);
                    joiner.finish(len);
                    pos = len;
                }
                continue;
            }

            // Tag start
            if byte_at(source, pos) == b'<' {
                if pos + 1 < len
                    && byte_at(source, pos + 1) == b'/'
                    && let Some((tag_name, end_pos)) = parse_closing_tag(input, pos)
                {
                    self.flush_text_buffer::<CANONICAL>(&mut output, &mut text, depth, &mut joiner);
                    let join = joiner.open(pos);
                    depth = depth.saturating_sub(1);
                    inline = InlineNode::None;
                    self.open_chunk(&mut output, depth, join);
                    output.push_str("</");
                    output.push_str(&tag_name);
                    // Prettier closes a hugged element whose opening `>` moved
                    // down with `>` on its own line too.
                    let moved_opening = joiner.take_moved_opening_bracket(depth);
                    if (CANONICAL && joiner.take_wrapped_text_close(pos))
                        || (CANONICAL && moved_opening && join.is_continuation())
                        || (join.is_continuation()
                            && !joiner.locks_current_line()
                            && (self.closing_bracket_overflows(&output)
                                || self.following_text_overflows::<CANONICAL>(
                                    &output, input, end_pos,
                                )))
                    {
                        output.push_str(self.newline);
                        self.write_indent(&mut output, depth);
                    }
                    output.push('>');
                    output.push_str(self.newline);
                    joiner.finish(end_pos);
                    pos = end_pos;
                    continue;
                }
                if let Some((tag_name, attrs, is_self_closing, end_pos)) =
                    self.parse_opening_tag(input, pos, depth)
                {
                    self.flush_text_buffer::<CANONICAL>(&mut output, &mut text, depth, &mut joiner);
                    let join = joiner.open(pos);
                    let mut sorted_attrs = attrs;
                    inline = InlineNode::None;
                    if self.options.sort_attributes {
                        sort_attributes(&tag_name, &mut sorted_attrs, self.options);
                    }
                    if self.options.normalize_inline_styles {
                        normalize_inline_styles(&mut sorted_attrs);
                    }

                    // Compute once per opening tag; consumed in the two
                    // void-element branches below.
                    let is_void = is_void_element_str(&tag_name);
                    let is_self_closing =
                        is_self_closing || (is_void && self.options.self_close_void_elements);
                    // The policy is pure; ordinary nonempty tags previously
                    // queried it twice. Self-closing tags never query it.
                    let whitespace_significant = !is_self_closing
                        && is_whitespace_significant_element(&tag_name, &sorted_attrs);
                    let hugged_inline_suffix = if is_self_closing || joiner.locks_current_line() {
                        None
                    } else {
                        self.hugged_inline_suffix_width::<CANONICAL>(
                            input,
                            end_pos,
                            &tag_name,
                            depth,
                            !is_void && !whitespace_significant,
                        )
                    };
                    // An empty element's `></tag>` shares the start-tag line.
                    let empty_close = if is_self_closing || is_void {
                        None
                    } else {
                        self.empty_element_close(input, end_pos, &tag_name, whitespace_significant)
                    };
                    let compact_suffix_width = match empty_close {
                        Some(_) => 4 + tag_name.width(),
                        None if is_self_closing => 3,
                        None => 1,
                    };
                    let continued = if !join.is_continuation() || joiner.locks_current_line() {
                        Continued::No
                    } else if joiner.follows_opening_bracket(pos) {
                        Continued::AfterOpeningBracket
                    } else {
                        Continued::Line
                    };
                    self.open_chunk(&mut output, depth, join);
                    output.push('<');
                    output.push_str(&tag_name);

                    let line = self.write_opening_attributes_with_prefix(
                        &mut output,
                        &tag_name,
                        &sorted_attrs,
                        depth,
                        continued,
                        compact_suffix_width,
                    );
                    if line.moved_parent_bracket {
                        joiner.moved_opening_bracket(depth.saturating_sub(1));
                    }
                    let mut closing_bracket_on_own_line = line.bracket_on_own_line;
                    if !closing_bracket_on_own_line
                        && hugged_inline_suffix.is_some_and(|suffix| {
                            self.hugged_inline_suffix_overflows(&output, suffix)
                        })
                    {
                        output.push_str(self.newline);
                        self.write_indent(&mut output, depth);
                        closing_bracket_on_own_line = true;
                    }

                    if is_self_closing {
                        if closing_bracket_on_own_line {
                            output.push_str("/>");
                        } else {
                            output.push_str(" />");
                        }
                    } else if !whitespace_significant && let Some(closing_end_pos) = empty_close {
                        output.push('>');
                        output.push_str("</");
                        output.push_str(&tag_name);
                        output.push('>');
                        output.push_str(self.newline);
                        joiner.finish(closing_end_pos);
                        pos = closing_end_pos;
                        continue;
                    } else if whitespace_significant {
                        // Copy `<pre>`/`<textarea>`/`v-pre` content verbatim so
                        // the formatter never changes rendered output.
                        // (#963, #3249)
                        pos = self.copy_whitespace_significant_element(
                            input,
                            end_pos,
                            &tag_name,
                            len,
                            &mut output,
                        );
                        joiner.finish(pos);
                        continue;
                    } else {
                        output.push('>');
                        if !is_void {
                            nesting::open_element(depth)?;
                            depth += 1;
                            joiner.opened_element(pos, end_pos, tag_name.len());
                        }
                    }
                    output.push_str(self.newline);
                    joiner.finish(end_pos);
                    pos = end_pos;
                    continue;
                }
                // Keep a non-tag `<` as text and advance past it.
                text.push_char(pos, '<');
                if CANONICAL {
                    inline = inline.track(source, pos, pos + 1);
                }
                pos += 1;
                continue;
            }

            // Accumulate text content until newline or tag
            let content_start = pos;
            while pos < len {
                let tail = sub_slice(source, pos..);
                let primary = memchr3(b'\n', b'<', b'{', tail);
                let offset = if HAS_CR {
                    earliest_offset(primary, memchr::memchr(b'\r', tail))
                } else {
                    primary
                };
                let Some(offset) = offset else {
                    pos = len;
                    break;
                };
                pos += offset;

                match byte_at(source, pos) {
                    b'\n' | b'<' => break,
                    b'\r' if HAS_CR => break,
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
                    text.push_source(input, content_start, content_end);
                    if CANONICAL {
                        inline = inline.track(source, content_start, content_end);
                    }
                }
            }

            // Handle newline
            if pos < len
                && (byte_at(source, pos) == b'\n' || (HAS_CR && byte_at(source, pos) == b'\r'))
            {
                // Suppression pragmas own physical lines; ordinary condensed
                // text instead forms one canonical run across authored wraps,
                // except where Prettier keeps an inline node on its own line.
                let kept = if CANONICAL && !text.is_empty() {
                    inline.kept_line_break(source, pos)
                } else {
                    None
                };
                if !CANONICAL || joiner.has_locked_lines() || kept.is_some() {
                    self.flush_text_buffer::<CANONICAL>(&mut output, &mut text, depth, &mut joiner);
                }
                if let Some((next, true)) = kept {
                    joiner.keep_blank_line(next);
                }
                pos += 1;
            }
        }

        // Flush remaining content
        self.flush_text_buffer::<CANONICAL>(&mut output, &mut text, depth, &mut joiner);

        // Remove trailing newline for consistency
        let kept = output.trim_end_matches(['\n', '\r']).len();
        output.truncate(kept);

        Ok(output.into_compact())
    }

    #[inline]
    fn write_indent(&self, output: &mut TextBuf, depth: usize) {
        for _ in 0..depth {
            output.push_str(self.indent);
        }
    }

    #[inline]
    fn write_indented_line(&self, output: &mut TextBuf, content: &str, depth: usize) {
        self.write_indent(output, depth);
        output.push_str(content);
        output.push_str(self.newline);
    }
}

/// The text after the last line break of `output`.
#[inline]
pub(crate) fn current_line(output: &str) -> &str {
    let start = memchr::memrchr2(b'\r', b'\n', output.as_bytes()).map_or(0, |pos| pos + 1);
    output.get(start..).unwrap_or_default()
}

fn earliest_offset(left: Option<usize>, right: Option<usize>) -> Option<usize> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (offset, None) | (None, offset) => offset,
    }
}

fn normalize_inline_styles(attrs: &mut [super::attributes::ParsedAttribute]) {
    for attr in attrs.iter_mut().filter(|attr| attr.name == "style") {
        if let Some(normalized) = attr
            .value
            .as_deref()
            .and_then(super::inline_style::normalize)
        {
            attr.value = Some(normalized);
        }
    }
}
