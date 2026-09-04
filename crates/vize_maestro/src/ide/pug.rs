//! Lightweight authored-Pug template cursor helpers.

pub(crate) struct OpeningTagContext {
    pub(crate) tag_name: String,
    pub(crate) tag_start: usize,
    pub(crate) current_token: String,
    pub(crate) current_token_start: usize,
    pub(crate) inside_attribute_value: bool,
}

pub(crate) fn tag_name_span_at_offset(
    content: &str,
    offset: usize,
) -> Option<(usize, usize, usize, usize)> {
    let cursor = offset.min(content.len());
    if !is_in_pug_template_region(content, cursor) {
        return None;
    }

    let line_start = content[..cursor].rfind('\n').map_or(0, |index| index + 1);
    let line_end = content[cursor..]
        .find('\n')
        .map_or(content.len(), |index| cursor + index);
    let bytes = content.as_bytes();
    let mut name_start = line_start;

    while name_start < line_end && matches!(bytes[name_start], b' ' | b'\t') {
        name_start += 1;
    }
    if matches!(
        bytes.get(name_start),
        None | Some(b'.' | b'#' | b'/' | b'|' | b':' | b'-' | b'+')
    ) {
        return None;
    }

    let name_end = read_tag_name_end(content, name_start, line_end);
    (name_start != name_end).then_some((name_start, line_end, name_start, name_end))
}

pub(crate) fn opening_tag_context_at_offset(
    content: &str,
    offset: usize,
) -> Option<OpeningTagContext> {
    let cursor = offset.min(content.len());
    let (_, line_end, tag_start, name_end) = tag_name_span_at_offset(content, cursor)?;
    if cursor < name_end {
        return None;
    }

    let bytes = content.as_bytes();
    let suffix_end = read_tag_suffix_end(content, name_end, line_end);
    let (attribute_start, attribute_end) = if bytes.get(suffix_end) == Some(&b'(') {
        let attribute_start = suffix_end + 1;
        let attribute_end = find_attribute_end(content, suffix_end, line_end)?;
        (attribute_start, attribute_end)
    } else {
        (suffix_end, suffix_end)
    };

    if cursor > attribute_end {
        return None;
    }

    let inside_attribute_value =
        cursor > attribute_start && is_inside_attribute_value(content, attribute_start, cursor);
    let (current_token_start, current_token) =
        current_attribute_token(content, attribute_start, cursor);

    Some(OpeningTagContext {
        tag_name: content[tag_start..name_end].to_string(),
        tag_start,
        current_token,
        current_token_start,
        inside_attribute_value,
    })
}

fn read_tag_name_end(content: &str, mut pos: usize, line_end: usize) -> usize {
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

fn read_tag_suffix_end(content: &str, mut pos: usize, line_end: usize) -> usize {
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

fn find_attribute_end(content: &str, open: usize, line_end: usize) -> Option<usize> {
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

fn is_inside_attribute_value(content: &str, attribute_start: usize, cursor: usize) -> bool {
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

fn current_attribute_token(
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

#[cfg(test)]
mod tests {
    use super::{opening_tag_context_at_offset, tag_name_span_at_offset};

    #[test]
    fn reads_pug_component_tag_names() {
        let source = r#"<template lang="pug">
  highlight-message(type="success")
</template>
"#;
        let offset = source.find("highlight-message").unwrap();

        assert_eq!(
            tag_name_span_at_offset(source, offset).map(|(_, _, start, end)| &source[start..end]),
            Some("highlight-message")
        );
    }

    #[test]
    fn opening_context_detects_class_shorthand_attribute_list() {
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
    fn opening_context_marks_attribute_values() {
        let source = r#"<template lang="pug">
  highlight-message(type="success")
</template>
"#;
        let offset = source.find("success").unwrap() + 2;
        let context = opening_tag_context_at_offset(source, offset).expect("Pug tag context");

        assert!(context.inside_attribute_value);
    }

    #[test]
    fn ignores_plain_html_template_text() {
        let source = r#"<template>
  highlight-message text
</template>
"#;
        let offset = source.find("highlight-message").unwrap();

        assert!(tag_name_span_at_offset(source, offset).is_none());
        assert!(opening_tag_context_at_offset(source, offset).is_none());
    }
}
