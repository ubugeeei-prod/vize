//! Existing opening-tag and attribute parsing.

use super::TemplateFormatter;
use crate::template::{
    attributes::ParsedAttribute,
    directives::normalize_attribute_with_vue_version,
    helpers::{byte_at, is_tag_name_char, is_whitespace, parse_closing_tag},
};
use vize_l0::{String, ToCompactString};
use vize_l1::markup::{
    DirectivePrefix, DirectiveSyntax, VueDirectives, lex::dynamic_argument_boundary,
};

impl TemplateFormatter<'_> {
    /// Parse an opening tag into structured attributes.
    pub(super) fn parse_opening_tag(
        &self,
        text: &str,
        start: usize,
        depth: usize,
    ) -> Option<(String, Vec<ParsedAttribute>, bool, usize)> {
        let source = text.as_bytes();
        let len = source.len();
        let mut pos = start + 1; // Skip '<'

        // Parse tag name
        let tag_start = pos;
        while pos < len && is_tag_name_char(byte_at(source, pos)) {
            pos += 1;
        }
        if pos == tag_start {
            return None;
        }

        // Every range below starts and ends at ASCII bytes of UTF-8 text, so
        // slicing the text needs no UTF-8 revalidation.
        let tag_name = text
            .get(tag_start..pos)
            .unwrap_or_default()
            .to_compact_string();

        // Parse attributes
        let mut attrs = Vec::new();
        let mut is_self_closing = false;
        let mut attr_index: usize = 0;

        while pos < len && byte_at(source, pos) != b'>' {
            // Skip whitespace
            while pos < len && is_whitespace(byte_at(source, pos)) {
                pos += 1;
            }
            if pos >= len {
                break;
            }

            // Check for self-closing or end
            if byte_at(source, pos) == b'/' {
                is_self_closing = true;
                pos += 1;
                continue;
            }
            if byte_at(source, pos) == b'>' {
                break;
            }

            // Parse single attribute
            let (attr, new_pos) = self.parse_single_attribute(text, pos, attr_index, depth);
            if let Some(attr) = attr {
                attrs.push(attr);
                attr_index += 1;
            }
            pos = new_pos;
        }

        // Skip '>'
        if pos < len && byte_at(source, pos) == b'>' {
            pos += 1;
        }

        Some((tag_name, attrs, is_self_closing, pos))
    }

    /// Return the end of an immediately following matching closing tag.
    pub(super) fn parse_immediate_empty_closing_tag(
        &self,
        text: &str,
        start: usize,
        tag_name: &str,
    ) -> Option<usize> {
        let source = text.as_bytes();
        let len = source.len();
        let mut pos = start;

        while pos < len && is_whitespace(byte_at(source, pos)) {
            pos += 1;
        }

        if pos + 1 >= len || byte_at(source, pos) != b'<' || byte_at(source, pos + 1) != b'/' {
            return None;
        }

        let (closing_tag_name, end_pos) = parse_closing_tag(text, pos)?;
        if closing_tag_name.as_str() == tag_name {
            Some(end_pos)
        } else {
            None
        }
    }

    /// Parse a single attribute: name, optional `="value"`.
    fn parse_single_attribute(
        &self,
        text: &str,
        start: usize,
        index: usize,
        depth: usize,
    ) -> (Option<ParsedAttribute>, usize) {
        let source = text.as_bytes();
        let len = source.len();
        let mut pos = start;

        // Parse attribute name (may include :, @, #, ., v-, etc.)
        let name_start = pos;
        while pos < len {
            let b = byte_at(source, pos);
            // The compiler's L1 lexer owns dynamic-argument boundaries. In
            // particular, an arrow's `=>` belongs to this name, not the HTML
            // assignment/tag end. Plain names and modifier text keep their
            // original framing; this does not parse or normalize the argument.
            if b == b'['
                && let Some(prefix) = text.get(name_start..pos)
                && let Ok(Some(head)) = VueDirectives.decompose(prefix, 0)
                && head.arg.is_none()
                && head.modifiers.start == head.modifiers.end
                && (head.prefix != DirectivePrefix::Full
                    || head.name.start != head.name.end && prefix.ends_with(':'))
            {
                let (end, closed) =
                    dynamic_argument_boundary(text.get(pos + 1..).unwrap_or_default());
                pos += 1 + end + usize::from(closed);
                if !closed {
                    break;
                }
                continue;
            }
            if is_whitespace(b) || b == b'>' || b == b'/' || b == b'=' {
                break;
            }
            pos += 1;
        }

        if pos == name_start {
            // Skip unknown byte to avoid infinite loop
            return (None, pos + 1);
        }

        let raw_name = text
            .get(name_start..pos)
            .unwrap_or_default()
            .to_compact_string();

        // Skip whitespace before '='
        let mut val_pos = pos;
        while val_pos < len && matches!(byte_at(source, val_pos), b' ' | b'\t') {
            val_pos += 1;
        }

        // Check for '=' and value
        let value = if val_pos < len && byte_at(source, val_pos) == b'=' {
            val_pos += 1; // skip '='

            // Skip whitespace after '='
            while val_pos < len && matches!(byte_at(source, val_pos), b' ' | b'\t') {
                val_pos += 1;
            }

            if val_pos < len && matches!(byte_at(source, val_pos), b'"' | b'\'') {
                // Quoted value
                let quote = byte_at(source, val_pos);
                val_pos += 1;
                let value_start = val_pos;
                while val_pos < len && byte_at(source, val_pos) != quote {
                    val_pos += 1;
                }
                let value = text
                    .get(value_start..val_pos)
                    .unwrap_or_default()
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
                    && !is_whitespace(byte_at(source, val_pos))
                    && byte_at(source, val_pos) != b'>'
                    && byte_at(source, val_pos) != b'/'
                {
                    val_pos += 1;
                }
                let value = text
                    .get(value_start..val_pos)
                    .unwrap_or_default()
                    .to_compact_string();
                pos = val_pos;
                Some(value)
            }
        } else {
            // Boolean attribute (no value)
            None
        };

        // Normalize directives and determine priority
        let (name, value, priority, indent_multiline_value, owns_value_lines) =
            normalize_attribute_with_vue_version(
                &raw_name,
                value,
                self.options,
                self.vue_version,
                depth + self.base_depth,
            );

        (
            Some(ParsedAttribute {
                name,
                value,
                priority,
                original_index: index,
                indent_multiline_value,
                owns_value_lines,
            }),
            pos,
        )
    }
}
