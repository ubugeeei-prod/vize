//! Lightweight authored-Pug template cursor helpers.

pub(crate) struct OpeningTagContext {
    pub(crate) tag_name: String,
    pub(crate) tag_start: usize,
    pub(crate) current_token: String,
    pub(crate) current_token_start: usize,
    pub(crate) inside_attribute_value: bool,
}

/// Authored byte spans for the Pug tag and attribute name under the cursor.
///
/// The attribute span includes Vue shorthand prefixes and event modifiers. It
/// is intentionally syntax-only so definition, hover, references, and rename
/// can apply their own semantic normalization to the same authored token.
pub(crate) struct AttributeNameSpan {
    pub(crate) tag_start: usize,
    pub(crate) tag_end: usize,
    pub(crate) attribute_start: usize,
    pub(crate) attribute_end: usize,
}

pub(crate) fn attribute_name_span_at_offset(
    content: &str,
    offset: usize,
) -> Option<AttributeNameSpan> {
    let cursor = offset.min(content.len());
    let (_, region_end, tag_start, tag_end) = tag_name_span_at_offset(content, cursor)?;
    let open = read_tag_suffix_end(content, tag_end, region_end);
    if content.as_bytes().get(open) != Some(&b'(') {
        return None;
    }
    let close = find_attribute_end(content, open, region_end)?;
    let mut pos = open + 1;

    while pos < close {
        pos = skip_attribute_separators(content, pos, close);
        if pos >= close {
            break;
        }

        let attribute_start = pos;
        while pos < close {
            let ch = content.get(pos..)?.chars().next()?;
            if ch.is_ascii_whitespace() || matches!(ch, '=' | ',' | ')') {
                break;
            }
            pos += ch.len_utf8();
        }
        let attribute_end = pos;
        if attribute_start == attribute_end {
            return None;
        }

        if cursor >= attribute_start && cursor <= attribute_end {
            return Some(AttributeNameSpan {
                tag_start,
                tag_end,
                attribute_start,
                attribute_end,
            });
        }

        pos = skip_ascii_whitespace(content, pos, close);
        if content.as_bytes().get(pos) == Some(&b'=') {
            pos = skip_attribute_value(content, pos + 1, close);
        }
    }

    None
}

fn skip_attribute_separators(content: &str, mut pos: usize, end: usize) -> usize {
    while pos < end
        && content
            .as_bytes()
            .get(pos)
            .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b',')
    {
        pos += 1;
    }
    pos
}

fn skip_ascii_whitespace(content: &str, mut pos: usize, end: usize) -> usize {
    while pos < end
        && content
            .as_bytes()
            .get(pos)
            .is_some_and(u8::is_ascii_whitespace)
    {
        pos += 1;
    }
    pos
}

fn skip_attribute_value(content: &str, mut pos: usize, end: usize) -> usize {
    pos = skip_ascii_whitespace(content, pos, end);
    let value_start = pos;
    let mut quote = None;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;

    while pos < end {
        let Some(ch) = content.get(pos..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        if let Some(open_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == open_quote {
                quote = None;
            }
        } else {
            match ch {
                '"' | '\'' | '`' => quote = Some(ch),
                '(' => paren_depth += 1,
                ')' if paren_depth > 0 => paren_depth -= 1,
                '[' => bracket_depth += 1,
                ']' if bracket_depth > 0 => bracket_depth -= 1,
                '{' => brace_depth += 1,
                '}' if brace_depth > 0 => brace_depth -= 1,
                ',' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => break,
                _ if ch.is_ascii_whitespace()
                    && paren_depth == 0
                    && bracket_depth == 0
                    && brace_depth == 0 =>
                {
                    let next = skip_ascii_whitespace(content, pos + ch.len_utf8(), end);
                    if looks_like_attribute_start(content, next, end)
                        && !starts_with_expression_continuation(content, next, end)
                        && is_complete_attribute_expression(content, value_start, pos)
                    {
                        return next;
                    }
                }
                _ => {}
            }
        }
        pos += ch.len_utf8();
    }

    pos
}

fn looks_like_attribute_start(content: &str, start: usize, end: usize) -> bool {
    let Some(first) = content.as_bytes().get(start) else {
        return false;
    };
    if !first.is_ascii_alphabetic() && !matches!(first, b'_' | b':' | b'@' | b'#') {
        return false;
    }

    let mut pos = start;
    while pos < end {
        let Some(ch) = content.get(pos..).and_then(|rest| rest.chars().next()) else {
            return false;
        };
        if ch.is_ascii_whitespace() || matches!(ch, '=' | ',' | ')') {
            break;
        }
        if !ch.is_ascii_alphanumeric()
            && !matches!(ch, '_' | '-' | '$' | ':' | '@' | '#' | '.' | '[' | ']')
        {
            return false;
        }
        pos += ch.len_utf8();
    }
    start < pos
}

fn starts_with_expression_continuation(content: &str, start: usize, end: usize) -> bool {
    let Some(rest) = content.get(start..end) else {
        return false;
    };
    let token_end = rest
        .find(|ch: char| !ch.is_ascii_alphabetic())
        .unwrap_or(rest.len());
    matches!(
        rest.get(..token_end),
        Some("in" | "instanceof" | "as" | "satisfies")
    )
}

fn is_complete_attribute_expression(content: &str, value_start: usize, value_end: usize) -> bool {
    let Some(value) = content.get(value_start..value_end).map(str::trim) else {
        return false;
    };
    if value.is_empty() {
        return false;
    }

    let allocator = oxc_allocator::Allocator::default();
    oxc_parser::Parser::new(&allocator, value, oxc_span::SourceType::ts())
        .parse_expression()
        .is_ok()
}

pub(crate) fn tag_name_span_at_offset(
    content: &str,
    offset: usize,
) -> Option<(usize, usize, usize, usize)> {
    let cursor = offset.min(content.len());
    let (body_start, body_end) = pug_template_body_bounds(content, cursor)?;
    let current_line_start = content
        .get(..cursor)?
        .rfind('\n')
        .map_or(body_start, |index| (index + 1).max(body_start));
    let mut line_start = body_start;

    while line_start <= current_line_start && line_start < body_end {
        let Some((_, line_end, tag_start, name_end)) = tag_name_span_on_line(content, line_start)
        else {
            line_start = next_line_start(content, line_start, body_end)?;
            continue;
        };
        let open = read_tag_suffix_end(content, name_end, line_end);
        if content.as_bytes().get(open) == Some(&b'(') {
            if let Some(attribute_end) = find_attribute_end(content, open, body_end) {
                if line_start == current_line_start || (open < cursor && cursor <= attribute_end) {
                    return Some((
                        line_start,
                        attribute_end.saturating_add(1),
                        tag_start,
                        name_end,
                    ));
                }
                line_start = next_line_start(content, attribute_end, body_end)?;
                continue;
            }
        } else if line_start == current_line_start {
            return Some((line_start, line_end, tag_start, name_end));
        }

        line_start = next_line_start(content, line_end, body_end)?;
    }

    None
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
        tag_name: content.get(tag_start..name_end)?.to_string(),
        tag_start,
        current_token,
        current_token_start,
        inside_attribute_value,
    })
}

/// Start of the Vue expression (`:prop`, `@event`, `#slot`, `v-*`) whose quoted
/// value in a Pug attribute list (`button(@click="inc")`) holds the cursor.
pub(crate) fn directive_value_start_at_offset(content: &str, offset: usize) -> Option<usize> {
    let cursor = offset.min(content.len());
    let (_, line_end, _, name_end) = tag_name_span_at_offset(content, cursor)?;
    let open = read_tag_suffix_end(content, name_end, line_end);
    if content.as_bytes().get(open) != Some(&b'(')
        || cursor > find_attribute_end(content, open, line_end)?
    {
        return None;
    }
    let mut quote = None;
    let mut escaped = false;
    let mut name_start = open + 1;
    let mut value_start = None;
    let mut pos = open + 1;
    while pos < cursor {
        let ch = content.get(pos..)?.chars().next()?;
        if let Some(open_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == open_quote {
                quote = None;
                value_start = None;
            }
        } else if matches!(ch, '"' | '\'' | '`') {
            quote = Some(ch);
            value_start = Some(pos + 1);
        } else if ch.is_ascii_whitespace() || ch == ',' {
            name_start = pos + 1;
        }
        pos += ch.len_utf8();
    }
    let value_start = value_start?;
    let name = content.get(name_start..value_start.checked_sub(1)?)?;
    let name = name.trim().strip_suffix('=')?.trim_end();
    (name.starts_with([':', '@', '#']) || name.starts_with("v-")).then_some(value_start)
}

fn read_tag_name_end(content: &str, mut pos: usize, line_end: usize) -> usize {
    while pos < line_end
        && content
            .as_bytes()
            .get(pos)
            .is_some_and(|&byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        pos += 1;
    }
    pos
}

fn read_tag_suffix_end(content: &str, mut pos: usize, line_end: usize) -> usize {
    let bytes = content.as_bytes();
    while pos < line_end && matches!(bytes.get(pos), Some(b'.' | b'#')) {
        pos = read_tag_name_end(content, pos + 1, line_end);
    }
    pos
}

fn find_attribute_end(content: &str, open: usize, search_end: usize) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    let mut depth = 0usize;
    let mut pos = open;

    while pos < search_end {
        let ch = content.get(pos..)?.chars().next()?;
        if let Some(open_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == open_quote {
                quote = None;
            }
        } else if matches!(ch, '"' | '\'' | '`') {
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
    let mut escaped = false;
    let mut pos = attribute_start;

    while pos < cursor {
        let Some(ch) = content.get(pos..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        if let Some(open_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == open_quote {
                quote = None;
            }
        } else if matches!(ch, '"' | '\'' | '`') {
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
    let mut escaped = false;
    let mut pos = attribute_start;

    while pos < cursor {
        let Some(ch) = content.get(pos..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        if let Some(open_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == open_quote {
                quote = None;
            }
        } else if matches!(ch, '"' | '\'' | '`') {
            quote = Some(ch);
        } else if ch.is_ascii_whitespace() || ch == ',' || ch == '(' {
            token_start = pos + ch.len_utf8();
        }
        pos += ch.len_utf8();
    }

    (
        token_start,
        content
            .get(token_start..cursor)
            .unwrap_or_default()
            .trim_start()
            .to_string(),
    )
}

fn tag_name_span_on_line(content: &str, line_start: usize) -> Option<(usize, usize, usize, usize)> {
    let line_end = content
        .get(line_start..)?
        .find('\n')
        .map_or(content.len(), |index| line_start + index);
    let bytes = content.as_bytes();
    let mut name_start = line_start;

    while name_start < line_end && matches!(bytes.get(name_start), Some(b' ' | b'\t')) {
        name_start += 1;
    }
    if matches!(
        bytes.get(name_start),
        None | Some(b'.' | b'#' | b'/' | b'|' | b':' | b'-' | b'+')
    ) {
        return None;
    }

    let name_end = read_tag_name_end(content, name_start, line_end);
    (name_start != name_end).then_some((line_start, line_end, name_start, name_end))
}

fn next_line_start(content: &str, offset: usize, body_end: usize) -> Option<usize> {
    content
        .get(offset..body_end)?
        .find('\n')
        .map(|index| offset + index + 1)
}

fn pug_template_body_bounds(content: &str, cursor: usize) -> Option<(usize, usize)> {
    let before = content.get(..cursor.min(content.len()))?;
    let template_start = before.rfind("<template")?;
    let template = content.get(template_start..)?;
    if before
        .get(template_start..)
        .is_some_and(|open| open.contains("</template"))
    {
        return None;
    }

    let tag_end = template.find('>')?;
    if cursor <= template_start + tag_end {
        return None;
    }

    if !template_start_tag_lang_is_pug(template.get(..=tag_end).unwrap_or_default()) {
        return None;
    }

    let body_start = template_start + tag_end + 1;
    let body_end = content
        .get(body_start..)?
        .find("</template")
        .map_or(content.len(), |index| body_start + index);
    (cursor <= body_end).then_some((body_start, body_end))
}

fn template_start_tag_lang_is_pug(start_tag: &str) -> bool {
    let lower = start_tag.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut search = 0usize;

    while let Some(relative) = lower.get(search..).and_then(|rest| rest.find("lang")) {
        let name_start = search + relative;
        let name_end = name_start + "lang".len();
        search = name_end;

        if name_start
            .checked_sub(1)
            .and_then(|prev| bytes.get(prev))
            .is_some_and(|&byte| is_attribute_name_byte(byte))
        {
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
            Some(&quote @ (b'"' | b'\'')) => {
                pos += 1;
                let value_start = pos;
                while bytes.get(pos).is_some_and(|byte| *byte != quote) {
                    pos += 1;
                }
                lower.get(value_start..pos).unwrap_or_default()
            }
            Some(_) => {
                let value_start = pos;
                while bytes
                    .get(pos)
                    .is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b'>')
                {
                    pos += 1;
                }
                lower.get(value_start..pos).unwrap_or_default()
            }
            None => continue,
        };

        if matches!(value, "pug" | "jade") {
            return true;
        }
    }

    false
}

fn is_attribute_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':')
}

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[cfg(test)]
mod tests {
    use super::{
        attribute_name_span_at_offset, directive_value_start_at_offset,
        opening_tag_context_at_offset, tag_name_span_at_offset,
    };

    #[test]
    fn reads_pug_and_jade_component_tag_names() {
        for language in ["pug", "jade", "JADE"] {
            let language = ["lang=\"", language, "\""].concat();
            let source = r#"<template lang="pug">
  highlight-message(type="success")
</template>
"#
            .replace("lang=\"pug\"", &language);
            let offset = source.find("highlight-message").unwrap();

            assert_eq!(
                tag_name_span_at_offset(&source, offset)
                    .map(|(_, _, start, end)| &source[start..end]),
                Some("highlight-message"),
                "{language}"
            );
        }
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
    fn reads_multiline_attribute_lists() {
        for language in ["pug", "jade"] {
            let language_attr = ["lang=\"", language, "\""].concat();
            let source = r#"<template lang="pug">
  pane.w-flex(
    type="plain"
    :size="format(\"close)\", nested(pane.size))"
    @click=`inc(")")`
  ) text
</template>
"#
            .replace("lang=\"pug\"", &language_attr);
            let token_offset = source.find(":size").unwrap() + ":si".len();
            let context = opening_tag_context_at_offset(&source, token_offset)
                .expect("multiline Pug tag context");
            assert_eq!(context.tag_name, "pane", "{language}");
            assert_eq!(context.current_token, ":si", "{language}");
            assert!(!context.inside_attribute_value, "{language}");

            for (needle, expected) in [
                (
                    "pane.size",
                    Some(r#"format(\"close)\", nested(pane.size))"#),
                ),
                ("inc(\")\")", Some("inc(\")\")")),
                ("plain", None),
            ] {
                let offset = source.find(needle).unwrap() + 1;
                let start = directive_value_start_at_offset(&source, offset);
                assert_eq!(
                    start.map(|start| &source[start..start + expected.map_or(0, str::len)]),
                    expected,
                    "{language}/{needle}"
                );
            }

            let value_offset = source.find("pane.size").unwrap() + 2;
            let value_context = opening_tag_context_at_offset(&source, value_offset)
                .expect("multiline Pug attribute value context");
            assert!(value_context.inside_attribute_value, "{language}");
            assert_eq!(
                tag_name_span_at_offset(&source, value_offset)
                    .map(|(_, _, start, end)| &source[start..end]),
                Some("pane"),
                "{language}"
            );
            let text_offset = source.find(") text").unwrap() + ") ".len() + 1;
            assert!(
                opening_tag_context_at_offset(&source, text_offset).is_none(),
                "{language}"
            );
        }
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

    #[test]
    fn directive_values_in_attribute_lists_are_expressions() {
        let source = "<template lang=\"pug\">\n  pane.w-flex(title=\"plain\", :size=\"pane.size\" @click='inc(1)' v-if=\"ok\") text\n</template>\n";
        for (needle, expected) in [
            ("pane.size", Some("pane.size")),
            ("inc(1)", Some("inc(1)")),
            ("ok\"", Some("ok")),
            ("plain", None),
            ("text", None),
        ] {
            let offset = source.find(needle).unwrap() + 1;
            let start = directive_value_start_at_offset(source, offset);
            assert_eq!(
                start.map(|start| &source[start..start + expected.map_or(0, str::len)]),
                expected,
                "{needle}"
            );
        }
        let html = "<template>\n  pane(:size=\"pane.size\")\n</template>\n";
        let offset = html.find("pane.size").unwrap() + 1;
        assert_eq!(directive_value_start_at_offset(html, offset), None);
    }

    #[test]
    fn reads_authored_attribute_name_spans_for_pug_and_jade() {
        for language in ["pug", "jade"] {
            let language_attr = ["lang=\"", language, "\""].concat();
            let source = r#"<template lang="pug">
  Child.card(
    :message="format(value, nested(1))"
    @save-item.once='save'
    plain
    :value=left + right
  )
</template>
"#
            .replace("lang=\"pug\"", &language_attr);

            for (needle, expected) in [
                (":message", ":message"),
                ("save-item", "@save-item.once"),
                ("plain", "plain"),
            ] {
                let offset = source.find(needle).unwrap() + 1;
                let span = attribute_name_span_at_offset(&source, offset)
                    .expect("authored Pug attribute name");
                assert_eq!(
                    &source[span.tag_start..span.tag_end],
                    "Child",
                    "{language}/{needle}"
                );
                assert_eq!(
                    &source[span.attribute_start..span.attribute_end],
                    expected,
                    "{language}/{needle}"
                );
            }

            let value_offset = source.find("nested").unwrap() + 1;
            assert!(attribute_name_span_at_offset(&source, value_offset).is_none());
            let spaced_value_offset = source.find("right").unwrap() + 1;
            assert!(
                attribute_name_span_at_offset(&source, spaced_value_offset).is_none(),
                "{language}: an identifier inside an unquoted expression is not an attribute name",
            );
        }
    }

    #[test]
    fn separates_bare_attributes_from_complete_unquoted_expressions() {
        for language in ["pug", "jade"] {
            let language_attr = ["lang=\"", language, "\""].concat();
            let source = r#"<template lang="pug">
  Child(
    :model=model disabled
    :binary=left + right required
    :construct=new Foo readonly
    :unary=typeof value autofocus
    :keyword=left instanceof right multiple
  )
</template>
"#
            .replace("lang=\"pug\"", &language_attr);

            for name in ["disabled", "required", "readonly", "autofocus", "multiple"] {
                let offset = source.find(name).unwrap() + 1;
                let span = attribute_name_span_at_offset(&source, offset)
                    .expect("bare Pug attribute after an unquoted expression");
                assert_eq!(
                    source.get(span.attribute_start..span.attribute_end),
                    Some(name),
                    "{language}/{name}",
                );
            }

            for value_part in [
                "model disabled",
                "right required",
                "Foo readonly",
                "value autofocus",
                "instanceof",
                "right multiple",
            ] {
                let offset = source.find(value_part).unwrap() + 1;
                assert_eq!(
                    attribute_name_span_at_offset(&source, offset)
                        .and_then(|span| source.get(span.attribute_start..span.attribute_end)),
                    None,
                    "{language}/{value_part}",
                );
            }
        }
    }

    #[test]
    fn keeps_quoted_expression_suffixes_inside_attribute_values() {
        for language in ["pug", "jade"] {
            let language_attr = ["lang=\"", language, "\""].concat();
            let source = r#"<template lang="pug">
  Child(title='a' + b disabled)
</template>
"#
            .replace("lang=\"pug\"", &language_attr);

            let value_offset = source.find("b disabled").unwrap();
            assert!(
                attribute_name_span_at_offset(&source, value_offset).is_none(),
                "{language}: an identifier after a quoted value prefix is not an attribute name",
            );

            let attribute_offset = source.find("disabled").unwrap() + 1;
            let span = attribute_name_span_at_offset(&source, attribute_offset)
                .expect("bare Pug attribute after a quoted expression");
            assert_eq!(
                source.get(span.attribute_start..span.attribute_end),
                Some("disabled"),
                "{language}",
            );
        }
    }
}
