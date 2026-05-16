//! Helper utilities for the definition service.
//!
//! Provides word extraction, position conversion, import resolution,
//! and attribute inspection helpers.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use std::path::PathBuf;

use tower_lsp::lsp_types::Url;

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
    let mut line = 0u32;
    let mut col = 0u32;
    let mut current_offset = 0usize;

    for ch in content.chars() {
        if current_offset >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }

        current_offset += ch.len_utf8();
    }

    (line, col)
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
    let (_, _, name_start, name_end) = find_tag_name_span(content, cursor)?;

    if cursor < name_start || cursor > name_end {
        return None;
    }

    Some(content[name_start..name_end].to_string())
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

    let tag_name = &content[name_start..name_end];
    let mut pos = name_end;

    while pos < tag_end {
        while pos < tag_end && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        if pos >= tag_end || bytes[pos] == b'/' {
            break;
        }

        let attr_start = pos;
        while pos < tag_end
            && !bytes[pos].is_ascii_whitespace()
            && bytes[pos] != b'='
            && bytes[pos] != b'/'
        {
            pos += 1;
        }
        let attr_end = pos;

        if attr_start == attr_end {
            break;
        }

        let cursor_on_attr_name = cursor >= attr_start && cursor <= attr_end;
        let raw_attr_name = &content[attr_start..attr_end];

        while pos < tag_end && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        if pos < tag_end && bytes[pos] == b'=' {
            pos += 1;
            while pos < tag_end && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }

            if pos < tag_end && (bytes[pos] == b'"' || bytes[pos] == b'\'') {
                let quote = bytes[pos];
                pos += 1;
                while pos < tag_end && bytes[pos] != quote {
                    pos += 1;
                }
                if pos < tag_end {
                    pos += 1;
                }
            } else {
                while pos < tag_end && !bytes[pos].is_ascii_whitespace() && bytes[pos] != b'>' {
                    pos += 1;
                }
            }
        }

        if !cursor_on_attr_name {
            continue;
        }

        let mut attr_name = raw_attr_name;
        if let Some(stripped) = attr_name.strip_prefix(':') {
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

fn find_tag_name_span(content: &str, offset: usize) -> Option<(usize, usize, usize, usize)> {
    let bytes = content.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let mut cursor = offset.min(bytes.len());
    if cursor == bytes.len() {
        cursor = cursor.saturating_sub(1);
    }

    let mut tag_start = None;
    let mut i = cursor + 1;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b'<' => {
                tag_start = Some(i);
                break;
            }
            b'>' | b'\n' => return None,
            _ => {}
        }
    }

    let tag_start = tag_start?;
    let mut tag_end = tag_start;
    let mut quote = None;

    while tag_end < bytes.len() {
        let byte = bytes[tag_end];
        if let Some(current_quote) = quote {
            if byte == current_quote {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
        } else if byte == b'>' {
            break;
        } else if byte == b'\n' {
            return None;
        }
        tag_end += 1;
    }

    if tag_end >= bytes.len() || bytes[tag_end] != b'>' {
        return None;
    }

    let mut name_start = tag_start + 1;
    if name_start < tag_end && bytes[name_start] == b'/' {
        name_start += 1;
    }

    let mut name_end = name_start;
    while name_end < tag_end {
        let byte = bytes[name_end];
        if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' {
            name_end += 1;
        } else {
            break;
        }
    }

    if name_start == name_end {
        return None;
    }

    Some((tag_start, tag_end, name_start, name_end))
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
    #[allow(clippy::disallowed_macros)]
    let patterns = [
        format!("{}: ", property_name),
        format!("{}?: ", property_name),
        format!("{} :", property_name),
        format!("{}?:", property_name),
    ];

    for pattern in &patterns {
        if let Some(pos) = content.find(pattern.as_str()) {
            let before = &content[..pos];
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
    let content = &ctx.content;
    let offset = ctx.offset;

    // Check if we're inside a mustache expression {{ ... }}
    let before = &content[..offset];
    let after = &content[offset..];

    if let Some(mustache_start) = before.rfind("{{") {
        let between = &content[mustache_start + 2..offset];
        if !between.contains("}}") && after.contains("}}") {
            return true;
        }
    }

    // Check if we're inside an attribute value
    let mut pos = offset;
    let mut in_quotes = false;
    let mut quote_char = '"';

    while pos > 0 {
        let c = content.as_bytes()[pos - 1] as char;
        if c == '"' || c == '\'' {
            in_quotes = true;
            quote_char = c;
            pos -= 1;
            break;
        }
        if c == '>' || c == '<' {
            return false;
        }
        pos -= 1;
    }

    if !in_quotes {
        return false;
    }

    // Skip the = sign
    while pos > 0 && content.as_bytes()[pos - 1] == b'=' {
        pos -= 1;
    }

    // Get the attribute name by scanning backwards
    let attr_end = pos;
    while pos > 0 {
        let c = content.as_bytes()[pos - 1] as char;
        if c.is_whitespace() || c == '<' || c == '>' {
            break;
        }
        pos -= 1;
    }

    let attr_name = &content[pos..attr_end];

    if attr_name.starts_with(':')
        || attr_name.starts_with('@')
        || attr_name.starts_with('#')
        || attr_name.starts_with("v-")
    {
        let quote_start = attr_end + 1;
        if let Some(quote_end) = content[quote_start + 1..].find(quote_char) {
            let abs_quote_end = quote_start + 1 + quote_end;
            return offset <= abs_quote_end;
        }
    }

    false
}

/// Find the import path for a given component name.
pub(crate) fn find_import_path(ctx: &IdeContext<'_>, component_name: &str) -> Option<String> {
    let content = &ctx.content;

    // Pattern 1: import ComponentName from 'path'
    #[allow(clippy::disallowed_macros)]
    let default_import_pattern = format!("import {} from", component_name);
    if let Some(pos) = content.find(&default_import_pattern) {
        return extract_import_path_from_pos(content, pos + default_import_pattern.len());
    }

    // Pattern 2: import { ComponentName } from 'path'
    let import_positions: Vec<_> = content.match_indices("import ").collect();
    #[allow(clippy::disallowed_macros)]
    for (pos, _) in import_positions {
        let rest = &content[pos..];
        if let Some(from_pos) = rest.find(" from") {
            let import_clause = &rest[7..from_pos]; // Skip "import "
            if import_clause.contains(&format!("{{ {}", component_name))
                || import_clause.contains(&format!("{} }}", component_name))
                || import_clause.contains(&format!(", {}", component_name))
                || import_clause.contains(&format!("{},", component_name))
                || import_clause == format!("{{ {} }}", component_name)
            {
                return extract_import_path_from_pos(rest, from_pos + 5);
            }
        }
    }

    None
}

/// Extract import path from a position after 'from'.
pub(crate) fn extract_import_path_from_pos(content: &str, pos: usize) -> Option<String> {
    let rest = content[pos..].trim_start();

    let quote_char = rest.chars().next()?;
    if quote_char != '\'' && quote_char != '"' {
        return None;
    }

    let path_start = 1;
    let path_end = rest[path_start..].find(quote_char)?;

    Some(rest[path_start..path_start + path_end].to_string())
}

/// Resolve an import path relative to the current file.
pub(crate) fn resolve_import_path(current_uri: &Url, import_path: &str) -> Option<PathBuf> {
    let current_path = PathBuf::from(current_uri.path());
    let current_dir = current_path.parent()?;

    if import_path.starts_with("./") || import_path.starts_with("../") {
        let resolved = current_dir.join(import_path);

        if !resolved.exists() {
            let extensions = [".vue", ".ts", ".tsx", ".js", ".jsx"];
            for ext in extensions {
                let with_ext = resolved.with_extension(&ext[1..]);
                if with_ext.exists() {
                    return Some(with_ext);
                }
            }
            // Try index files
            for ext in extensions {
                #[allow(clippy::disallowed_macros)]
                let index_name = format!("index{}", ext);
                let index_file = resolved.join(index_name);
                if index_file.exists() {
                    return Some(index_file);
                }
            }
        }

        Some(resolved.canonicalize().unwrap_or(resolved))
    } else {
        None
    }
}
