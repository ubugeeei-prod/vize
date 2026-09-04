//! Opening-tag context detection, attribute-prefix predicates, and HTML tag
//! scanning helpers used by component prop/slot completions.

use crate::ide::is_component_tag;

#[derive(Debug)]
pub(super) struct OpenTagContext {
    pub tag_name: String,
    pub tag_start: usize,
    pub current_token: String,
    pub current_token_start: usize,
    pub inside_attribute_value: bool,
}

pub(super) fn opening_tag_context_at_offset(
    content: &str,
    offset: usize,
) -> Option<OpenTagContext> {
    html_opening_tag_context_at_offset(content, offset)
        .or_else(|| pug_opening_tag_context_at_offset(content, offset))
}

fn html_opening_tag_context_at_offset(content: &str, offset: usize) -> Option<OpenTagContext> {
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
    let (current_token_start, current_token) = current_open_tag_token(content, tag_start, cursor);

    Some(OpenTagContext {
        tag_name,
        tag_start,
        current_token,
        current_token_start,
        inside_attribute_value,
    })
}

fn pug_opening_tag_context_at_offset(content: &str, offset: usize) -> Option<OpenTagContext> {
    let cursor = offset.min(content.len());
    if !is_in_pug_template_region(content, cursor) {
        return None;
    }

    let line_start = content[..cursor].rfind('\n').map_or(0, |index| index + 1);
    let line_end = content[cursor..]
        .find('\n')
        .map_or(content.len(), |index| cursor + index);
    let bytes = content.as_bytes();
    let mut tag_start = line_start;

    while tag_start < line_end && matches!(bytes[tag_start], b' ' | b'\t') {
        tag_start += 1;
    }

    if matches!(
        bytes.get(tag_start),
        None | Some(b'.' | b'#' | b'/' | b'|' | b':' | b'-' | b'+')
    ) {
        return None;
    }

    let name_end = read_pug_tag_name_end(content, tag_start, line_end);
    if tag_start == name_end || cursor < name_end {
        return None;
    }

    let tag_suffix_end = read_pug_tag_suffix_end(content, name_end, line_end);
    let (attribute_start, attribute_end) = if bytes.get(tag_suffix_end) == Some(&b'(') {
        let attribute_start = tag_suffix_end + 1;
        let attribute_end = find_pug_attribute_end(content, tag_suffix_end, line_end)?;
        (attribute_start, attribute_end)
    } else {
        (tag_suffix_end, tag_suffix_end)
    };

    if cursor > attribute_end {
        return None;
    }

    let inside_attribute_value =
        cursor > attribute_start && is_inside_pug_attribute_value(content, attribute_start, cursor);
    let (current_token_start, current_token) =
        current_pug_attribute_token(content, attribute_start, cursor);

    Some(OpenTagContext {
        tag_name: content[tag_start..name_end].to_string(),
        tag_start,
        current_token,
        current_token_start,
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

fn is_inside_pug_attribute_value(content: &str, attribute_start: usize, cursor: usize) -> bool {
    let mut quote = None;
    let mut pos = attribute_start;

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

fn current_open_tag_token(content: &str, tag_start: usize, cursor: usize) -> (usize, String) {
    let slice = &content[tag_start..cursor];
    let mut token_start = tag_start;

    for (relative, ch) in slice.char_indices() {
        if ch.is_ascii_whitespace() || ch == '<' {
            token_start = tag_start + relative + ch.len_utf8();
        }
    }

    (
        token_start,
        content[token_start..cursor].trim_start().to_string(),
    )
}

fn current_pug_attribute_token(
    content: &str,
    attribute_start: usize,
    cursor: usize,
) -> (usize, String) {
    let mut token_start = cursor;
    let mut quote = None;
    let mut pos = attribute_start;

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
        } else if ch.is_ascii_whitespace() || ch == ',' || ch == '(' {
            token_start = pos + ch.len_utf8();
        }
        pos += ch.len_utf8();
    }

    (
        token_start,
        content[token_start..cursor].trim_start().to_string(),
    )
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

fn read_pug_tag_name_end(content: &str, mut pos: usize, line_end: usize) -> usize {
    while pos < line_end {
        let byte = content.as_bytes()[pos];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
            pos += 1;
        } else {
            break;
        }
    }
    pos
}

fn read_pug_tag_suffix_end(content: &str, mut pos: usize, line_end: usize) -> usize {
    let bytes = content.as_bytes();
    while pos < line_end {
        match bytes[pos] {
            b'.' | b'#' => {
                pos += 1;
                while pos < line_end {
                    let byte = bytes[pos];
                    if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
                        pos += 1;
                    } else {
                        break;
                    }
                }
            }
            _ => break,
        }
    }
    pos
}

fn find_pug_attribute_end(content: &str, open: usize, line_end: usize) -> Option<usize> {
    let mut quote = None;
    let mut depth = 0usize;
    let mut pos = open;

    while pos < line_end {
        let ch = content[pos..].chars().next()?;
        if let Some(open_quote) = quote {
            if ch == open_quote {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(pos);
            }
        }
        pos += ch.len_utf8();
    }

    None
}

fn is_in_pug_template_region(content: &str, cursor: usize) -> bool {
    let before = &content[..cursor.min(content.len())];
    let Some(template_start) = before.rfind("<template") else {
        return false;
    };

    if before[template_start..].rfind("</template").is_some() {
        return false;
    }

    let Some(tag_end) = content[template_start..].find('>') else {
        return false;
    };
    let tag_end = template_start + tag_end;
    if cursor <= tag_end {
        return false;
    }

    template_start_tag_lang_is_pug(&content[template_start..=tag_end])
}

fn template_start_tag_lang_is_pug(start_tag: &str) -> bool {
    let lower = start_tag.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut search = 0usize;

    while let Some(relative) = lower[search..].find("lang") {
        let name_start = search + relative;
        let name_end = name_start + "lang".len();
        search = name_end;

        if name_start > 0 && is_attribute_name_byte(bytes[name_start - 1]) {
            continue;
        }
        if bytes
            .get(name_end)
            .is_some_and(|byte| is_attribute_name_byte(*byte))
        {
            continue;
        }

        let mut pos = name_end;
        while bytes
            .get(pos)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'=') {
            continue;
        }
        pos += 1;
        while bytes
            .get(pos)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            pos += 1;
        }

        let value = match bytes.get(pos) {
            Some(b'"' | b'\'') => {
                let quote = bytes[pos];
                pos += 1;
                let value_start = pos;
                while bytes.get(pos).is_some_and(|byte| *byte != quote) {
                    pos += 1;
                }
                &lower[value_start..pos]
            }
            Some(_) => {
                let value_start = pos;
                while bytes
                    .get(pos)
                    .is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b'>')
                {
                    pos += 1;
                }
                &lower[value_start..pos]
            }
            None => continue,
        };

        if value == "pug" {
            return true;
        }
    }

    false
}

fn is_attribute_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':')
}

fn is_self_closing_tag(tag: &str) -> bool {
    tag.trim_end_matches('>').trim_end().ends_with('/')
}

#[cfg(test)]
mod tests {
    use super::opening_tag_context_at_offset;

    #[test]
    fn pug_context_detects_tag_attribute_boundary() {
        let source = r#"<template lang="pug">
  highlight-message(type="success")
</template>
"#;
        let offset = source.find("highlight-message").unwrap() + "highlight-message".len();
        let context = opening_tag_context_at_offset(source, offset).expect("Pug tag context");

        assert_eq!(context.tag_name, "highlight-message");
        assert_eq!(context.current_token, "");
        assert!(!context.inside_attribute_value);
    }

    #[test]
    fn pug_context_detects_class_shorthand_attribute_list() {
        let source = r#"<template lang="pug">
  pane.w-flex.align-center(:size="pane.size")
</template>
"#;
        let offset = source.find("(:size").unwrap() + 1;
        let context = opening_tag_context_at_offset(source, offset).expect("Pug tag context");

        assert_eq!(context.tag_name, "pane");
        assert_eq!(context.current_token, "");
        assert!(!context.inside_attribute_value);
    }

    #[test]
    fn pug_context_ignores_attribute_values() {
        let source = r#"<template lang="pug">
  highlight-message(type="success")
</template>
"#;
        let offset = source.find("success").unwrap() + 2;
        let context = opening_tag_context_at_offset(source, offset).expect("Pug tag context");

        assert!(context.inside_attribute_value);
    }

    #[test]
    fn pug_context_ignores_plain_html_template_text() {
        let source = r#"<template>
  highlight-message text
</template>
"#;
        let offset = source.find("highlight-message text").unwrap() + "highlight-message".len();

        assert!(opening_tag_context_at_offset(source, offset).is_none());
    }
}
