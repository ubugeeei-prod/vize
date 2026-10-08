//! Existing opening-tag and attribute parsing.

use super::TemplateFormatter;
use crate::template::{
    attributes::ParsedAttribute,
    directives::normalize_attribute_with_vue_version,
    helpers::{byte_at, is_tag_name_char, is_whitespace, parse_closing_tag, sub_slice},
};
use vize_l0::{String, ToCompactString};

impl TemplateFormatter<'_> {
    /// Parse an opening tag into structured attributes.
    pub(super) fn parse_opening_tag(
        &self,
        source: &[u8],
        start: usize,
        depth: usize,
    ) -> Option<(String, Vec<ParsedAttribute>, bool, usize)> {
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

        let tag_name = std::str::from_utf8(sub_slice(source, tag_start..pos))
            .unwrap_or("")
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
            let (attr, new_pos) = self.parse_single_attribute(source, pos, attr_index, depth);
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
        source: &[u8],
        start: usize,
        tag_name: &str,
    ) -> Option<usize> {
        let len = source.len();
        let mut pos = start;

        while pos < len && is_whitespace(byte_at(source, pos)) {
            pos += 1;
        }

        if pos + 1 >= len || byte_at(source, pos) != b'<' || byte_at(source, pos + 1) != b'/' {
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
        depth: usize,
    ) -> (Option<ParsedAttribute>, usize) {
        let len = source.len();
        let mut pos = start;

        // Parse attribute name (may include :, @, #, ., v-, etc.)
        let name_start = pos;
        while pos < len {
            let b = byte_at(source, pos);
            if is_whitespace(b) || b == b'>' || b == b'/' || b == b'=' {
                break;
            }
            pos += 1;
        }

        if pos == name_start {
            // Skip unknown byte to avoid infinite loop
            return (None, pos + 1);
        }

        let raw_name = std::str::from_utf8(sub_slice(source, name_start..pos))
            .unwrap_or("")
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
                let value = std::str::from_utf8(sub_slice(source, value_start..val_pos))
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
                    && !is_whitespace(byte_at(source, val_pos))
                    && byte_at(source, val_pos) != b'>'
                    && byte_at(source, val_pos) != b'/'
                {
                    val_pos += 1;
                }
                let value = std::str::from_utf8(sub_slice(source, value_start..val_pos))
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
        let value_indent = (depth + self.base_depth + 2) * self.options.tab_width as usize;
        let available_width = self
            .options
            .print_width
            .saturating_sub(value_indent as u32)
            .max(1);
        let (name, value, priority, indent_multiline_value, owns_value_lines) =
            normalize_attribute_with_vue_version(
                &raw_name,
                value,
                self.options,
                self.vue_version,
                available_width,
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
