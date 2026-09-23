//! Helper utilities for the definition service.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "definition helpers hand std `String` values to the lsp_types-based definition service"
)]

use std::path::PathBuf;

use tower_lsp::lsp_types::Url;
use vize_s0::cstr;

use super::IdeContext;

/// Get the word at a given offset.
pub(crate) fn get_word_at_offset(content: &str, offset: usize) -> Option<String> {
    crate::ide::token_at_offset(content, offset, is_word_char)
}

/// Check if a byte is a valid word character.
#[inline]
pub(crate) fn is_word_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

/// Convert byte offset to (line, character) position.
pub(crate) fn offset_to_position(content: &str, offset: usize) -> (u32, u32) {
    crate::ide::offset_to_position(content, offset)
}

/// Skip virtual code header comments.
pub(crate) fn skip_virtual_header(content: &str) -> usize {
    let mut offset = 0;
    for line in content.lines() {
        if line.starts_with("//") || line.trim().is_empty() {
            offset += line.len() + 1; // +1 for newline
        } else {
            break;
        }
    }
    offset
}

/// Get the tag name at the given offset (if cursor is on a tag).
pub(crate) fn get_tag_at_offset(content: &str, offset: usize) -> Option<String> {
    let cursor = offset.min(content.len());
    let (_, _, name_start, name_end) = find_tag_name_span(content, cursor)
        .or_else(|| crate::ide::pug::tag_name_span_at_offset(content, cursor))?;

    if cursor < name_start || cursor > name_end {
        return None;
    }

    content.get(name_start..name_end).map(str::to_string)
}

/// Get the attribute name and component name at the cursor position.
pub(crate) fn get_attribute_and_component_at_offset(
    ctx: &IdeContext<'_>,
) -> Option<(String, String)> {
    let content = &ctx.content;
    let cursor = ctx.offset.min(content.len());
    let (tag_start, tag_end, name_start, name_end) = find_tag_name_span(content, cursor)?;
    let bytes = content.as_bytes();

    if bytes.get(tag_start + 1) == Some(&b'/') {
        return None;
    }

    let tag_name = content.get(name_start..name_end)?;
    let mut pos = name_end;
    // Byte at `i` while still inside the tag; `None` past `tag_end`.
    let at = |i: usize| {
        if i < tag_end {
            bytes.get(i).copied()
        } else {
            None
        }
    };

    while pos < tag_end {
        while at(pos).is_some_and(|b| b.is_ascii_whitespace()) {
            pos += 1;
        }

        let Some(first) = at(pos) else { break };
        if first == b'/' {
            break;
        }

        let attr_start = pos;
        while at(pos).is_some_and(|b| !b.is_ascii_whitespace() && b != b'=' && b != b'/') {
            pos += 1;
        }
        let attr_end = pos;

        if attr_start == attr_end {
            break;
        }

        let cursor_on_attr_name = cursor >= attr_start && cursor <= attr_end;
        let raw_attr_name = content.get(attr_start..attr_end)?;

        while at(pos).is_some_and(|b| b.is_ascii_whitespace()) {
            pos += 1;
        }

        if at(pos) == Some(b'=') {
            pos += 1;
            while at(pos).is_some_and(|b| b.is_ascii_whitespace()) {
                pos += 1;
            }

            if let Some(quote @ (b'"' | b'\'')) = at(pos) {
                pos += 1;
                while at(pos).is_some_and(|b| b != quote) {
                    pos += 1;
                }
                if pos < tag_end {
                    pos += 1;
                }
            } else {
                while at(pos).is_some_and(|b| !b.is_ascii_whitespace() && b != b'>') {
                    pos += 1;
                }
            }
        }

        if !cursor_on_attr_name {
            continue;
        }

        let mut attr_name = raw_attr_name;
        if let Some(model_prop_name) =
            super::component_model::prop_name_from_v_model_attribute(raw_attr_name)
        {
            return Some((model_prop_name, tag_name.to_string()));
        } else if let Some(stripped) = attr_name.strip_prefix(':') {
            attr_name = stripped;
        } else if let Some(stripped) = attr_name.strip_prefix("v-bind:") {
            attr_name = stripped;
        } else if attr_name.starts_with('@')
            || attr_name.starts_with("v-on:")
            || attr_name.starts_with("v-")
        {
            return None;
        }

        if attr_name.is_empty() {
            return None;
        }

        return Some((attr_name.to_string(), tag_name.to_string()));
    }

    None
}

pub(crate) fn find_tag_name_span(
    content: &str,
    offset: usize,
) -> Option<(usize, usize, usize, usize)> {
    let bytes = content.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let mut cursor = offset.min(bytes.len());
    if cursor == bytes.len() {
        cursor = cursor.saturating_sub(1);
    }
    if cursor > 0 && bytes.get(cursor) == Some(&b'>') {
        cursor -= 1;
    }

    let mut search_end = cursor.saturating_add(1).min(content.len());
    while search_end > 0 {
        let tag_start = content.get(..search_end)?.rfind('<')?;
        let tag_end = find_tag_end_from(content, tag_start)?;

        if cursor > tag_end {
            return None;
        }

        let mut name_start = tag_start + 1;
        if name_start < tag_end && bytes.get(name_start) == Some(&b'/') {
            name_start += 1;
        }

        let mut name_end = name_start;
        while name_end < tag_end
            && bytes
                .get(name_end)
                .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            name_end += 1;
        }

        if name_start != name_end {
            return Some((tag_start, tag_end, name_start, name_end));
        }

        search_end = tag_start;
    }

    None
}

fn find_tag_end_from(content: &str, tag_start: usize) -> Option<usize> {
    let mut quote = None;

    for (tag_end, &byte) in content.as_bytes().iter().enumerate().skip(tag_start) {
        if let Some(current_quote) = quote {
            if byte == current_quote {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
        } else if byte == b'>' {
            return Some(tag_end);
        }
    }

    None
}

/// Convert kebab-case to camelCase.
pub(crate) fn kebab_to_camel(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Find a property name within defineProps type/object definition.
pub(crate) fn find_prop_in_define_props(content: &str, property_name: &str) -> Option<usize> {
    let patterns = [
        cstr!("{property_name}: "),
        cstr!("{property_name}?: "),
        cstr!("{property_name} :"),
        cstr!("{property_name}?:"),
    ];

    for pattern in &patterns {
        if let Some(pos) = content.find(pattern.as_str()) {
            let Some(before) = content.get(..pos) else {
                continue;
            };
            let open_angle = before.matches('<').count();
            let close_angle = before.matches('>').count();
            let open_curly = before.matches('{').count();
            let close_curly = before.matches('}').count();

            if open_angle > close_angle || open_curly > close_curly {
                return Some(pos);
            }
        }
    }

    None
}

/// Check if the cursor is inside a Vue directive expression.
pub(crate) fn is_in_vue_directive_expression(ctx: &IdeContext) -> bool {
    crate::ide::is_in_vue_template_expression(&ctx.content, ctx.offset)
}

/// Find the import path for a given component name.
pub(crate) fn find_import_path(ctx: &IdeContext<'_>, component_name: &str) -> Option<String> {
    let content = &ctx.content;

    // Pattern 1: import ComponentName from 'path'
    let default_import_pattern = cstr!("import {component_name} from");
    if let Some(pos) = content.find(default_import_pattern.as_str()) {
        return extract_import_path_from_pos(content, pos + default_import_pattern.len());
    }

    // Pattern 2: import { ComponentName } from 'path'
    let import_positions: Vec<_> = content.match_indices("import ").collect();
    for (pos, _) in import_positions {
        let Some(rest) = content.get(pos..) else {
            continue;
        };
        if let Some(from_pos) = rest.find(" from")
            && let Some(import_clause) = rest.get(7..from_pos)
        {
            // `import_clause` skips "import "
            if import_clause.contains(cstr!("{{ {component_name}").as_str())
                || import_clause.contains(cstr!("{component_name} }}").as_str())
                || import_clause.contains(cstr!(", {component_name}").as_str())
                || import_clause.contains(cstr!("{component_name},").as_str())
                || import_clause == cstr!("{{ {component_name} }}").as_str()
            {
                return extract_import_path_from_pos(rest, from_pos + 5);
            }
        }
    }

    None
}

/// Extract import path from a position after 'from'.
pub(crate) fn extract_import_path_from_pos(content: &str, pos: usize) -> Option<String> {
    let rest = content.get(pos..)?.trim_start();

    let quote_char = rest.chars().next()?;
    if quote_char != '\'' && quote_char != '"' {
        return None;
    }

    let path = rest.get(quote_char.len_utf8()..)?;
    let path_end = path.find(quote_char)?;

    path.get(..path_end).map(str::to_string)
}

/// Resolve a relative, package, or tsconfig-path import from the current file.
pub(crate) fn resolve_import_path(current_uri: &Url, import_path: &str) -> Option<PathBuf> {
    super::import_resolver::resolve_import_specifier(current_uri, import_path)
}
