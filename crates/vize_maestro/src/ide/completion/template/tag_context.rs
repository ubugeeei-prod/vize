//! Opening-tag context detection, attribute-prefix predicates, and HTML tag
//! scanning helpers used by component prop/slot completions.

use crate::ide::is_component_tag;

use super::ts_parse::skip_ws;

#[derive(Debug)]
pub(super) struct OpenTagContext {
    pub tag_name: String,
    pub tag_start: usize,
    pub current_token: String,
    pub inside_attribute_value: bool,
}

pub(super) fn opening_tag_context_at_offset(
    content: &str,
    offset: usize,
) -> Option<OpenTagContext> {
    let cursor = offset.min(content.len());
    let tag_start = content[..cursor].rfind('<')?;
    if content[tag_start..cursor].contains('>') {
        return None;
    }

    let bytes = content.as_bytes();
    let name_start = tag_start + 1;
    if matches!(bytes.get(name_start), Some(b'/' | b'!' | b'?')) {
        return None;
    }

    let mut name_end = name_start;
    while name_end < content.len() {
        let byte = bytes[name_end];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            name_end += 1;
        } else {
            break;
        }
    }

    if name_start == name_end || cursor <= name_end {
        return None;
    }

    let tag_name = content[name_start..name_end].to_string();
    let inside_attribute_value = is_inside_open_tag_attribute_value(content, tag_start, cursor);
    let current_token = current_open_tag_token(content, tag_start, cursor);

    Some(OpenTagContext {
        tag_name,
        tag_start,
        current_token,
        inside_attribute_value,
    })
}

fn is_inside_open_tag_attribute_value(content: &str, tag_start: usize, cursor: usize) -> bool {
    let mut quote = None;
    let mut pos = tag_start;

    while pos < cursor {
        let Some(ch) = content[pos..].chars().next() else {
            break;
        };
        if let Some(open_quote) = quote {
            if ch == open_quote {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        }
        pos += ch.len_utf8();
    }

    quote.is_some()
}

fn current_open_tag_token(content: &str, tag_start: usize, cursor: usize) -> String {
    let slice = &content[tag_start..cursor];
    let mut token_start = tag_start;

    for (relative, ch) in slice.char_indices() {
        if ch.is_ascii_whitespace() || ch == '<' {
            token_start = tag_start + relative + ch.len_utf8();
        }
    }

    content[token_start..cursor].trim_start().to_string()
}

pub(super) fn is_prop_completion_prefix(prefix: &str) -> bool {
    prefix.is_empty()
        || is_dynamic_prop_prefix(prefix)
        || (!prefix.starts_with('@')
            && !prefix.starts_with('#')
            && !prefix.starts_with("v-")
            && !prefix.contains('='))
}

pub(super) fn is_dynamic_prop_prefix(prefix: &str) -> bool {
    prefix.starts_with(':') || prefix.starts_with("v-bind:")
}

pub(super) fn is_slot_completion_prefix(prefix: &str) -> bool {
    prefix.is_empty() || prefix.starts_with('#') || prefix.starts_with("v-slot:")
}

pub(super) fn nearest_open_component_before(content: &str, before_offset: usize) -> Option<String> {
    let before = &content[..before_offset.min(content.len())];
    let mut stack = Vec::new();
    let mut pos = 0usize;

    while let Some(relative_start) = before[pos..].find('<') {
        let tag_start = pos + relative_start;
        if before[tag_start..].starts_with("<!--") {
            let Some(end) = before[tag_start + 4..].find("-->") else {
                break;
            };
            pos = tag_start + 4 + end + 3;
            continue;
        }

        let Some(tag_end) = find_tag_end(before, tag_start) else {
            break;
        };
        let tag = &before[tag_start..=tag_end];
        let name_start = tag_start + if tag.starts_with("</") { 2 } else { 1 };
        if matches!(before.as_bytes().get(name_start), Some(b'!' | b'?')) {
            pos = tag_end + 1;
            continue;
        }

        let name_end = read_tag_name_end(before, name_start);
        if name_start == name_end {
            pos = tag_end + 1;
            continue;
        }

        let tag_name = &before[name_start..name_end];
        if tag.starts_with("</") {
            if let Some(index) = stack.iter().rposition(|open: &String| open == tag_name) {
                stack.truncate(index);
            }
        } else if is_component_tag(tag_name) && !is_self_closing_tag(tag) {
            stack.push(tag_name.to_string());
        }

        pos = tag_end + 1;
    }

    stack.pop()
}

pub(super) fn find_tag_end(content: &str, tag_start: usize) -> Option<usize> {
    let mut quote = None;
    let mut pos = tag_start;

    while pos < content.len() {
        let ch = content[pos..].chars().next()?;
        if let Some(open_quote) = quote {
            if ch == open_quote {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '>' {
            return Some(pos);
        }
        pos += ch.len_utf8();
    }

    None
}

fn read_tag_name_end(content: &str, mut pos: usize) -> usize {
    while pos < content.len() {
        let byte = content.as_bytes()[pos];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            pos += 1;
        } else {
            break;
        }
    }
    pos
}

fn is_self_closing_tag(tag: &str) -> bool {
    tag.trim_end_matches('>').trim_end().ends_with('/')
}

pub(super) fn find_attr_value(tag: &str, attr: &str) -> Option<String> {
    let mut pos = 0usize;
    while let Some(relative) = tag[pos..].find(attr) {
        let start = pos + relative;
        let end = start + attr.len();
        let boundary_before = start == 0
            || tag
                .as_bytes()
                .get(start - 1)
                .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_');
        let boundary_after = tag
            .as_bytes()
            .get(end)
            .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_');
        if !boundary_before || !boundary_after {
            pos = end;
            continue;
        }

        let mut value_start = skip_ws(tag, end);
        if tag.as_bytes().get(value_start) != Some(&b'=') {
            pos = end;
            continue;
        }
        value_start = skip_ws(tag, value_start + 1);
        let quote = tag.as_bytes().get(value_start).copied()?;
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        let value_content_start = value_start + 1;
        let value_end = tag[value_content_start..].find(quote as char)? + value_content_start;
        return Some(tag[value_content_start..value_end].to_string());
    }

    None
}
