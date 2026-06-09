//! Low-level TypeScript type-literal scanning helpers shared by the component
//! metadata extractor. Approximate parsers that read prop/slot member names and
//! types out of `defineProps<{ … }>()` type arguments without a full TS parse.

pub(super) fn parse_type_literal_members(body: &str) -> Vec<&str> {
    let mut members = Vec::new();
    let mut start = 0usize;
    let mut state = SplitState::default();

    for (idx, ch) in body.char_indices() {
        state.accept(ch, body, idx);
        if state.depth == 0 && state.quote.is_none() && matches!(ch, ';' | ',' | '\n') {
            let member = body[start..idx].trim();
            if !member.is_empty() {
                members.push(member);
            }
            start = idx + ch.len_utf8();
        }
    }

    let member = body[start..].trim();
    if !member.is_empty() {
        members.push(member);
    }

    members
}

#[derive(Default)]
struct SplitState {
    depth: i32,
    quote: Option<char>,
}

impl SplitState {
    fn accept(&mut self, ch: char, source: &str, idx: usize) {
        if let Some(quote) = self.quote {
            if ch == quote && !is_escaped(source, idx) {
                self.quote = None;
            }
            return;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            self.quote = Some(ch);
        } else if matches!(ch, '{' | '[' | '(' | '<') {
            self.depth += 1;
        } else if matches!(ch, '}' | ']' | ')')
            || (ch == '>' && !previous_non_ws_is(source, idx, '='))
        {
            self.depth = self.depth.saturating_sub(1);
        }
    }
}

pub(super) fn parse_member_name_and_type(member: &str) -> Option<(String, bool, String)> {
    let member = strip_readonly(member.trim());
    let (name, name_end) = parse_type_member_name(member)?;
    let rest = member[name_end..].trim_start();
    let (optional, rest) = if let Some(rest) = rest.strip_prefix('?') {
        (true, rest.trim_start())
    } else {
        (false, rest)
    };
    let type_detail = rest.strip_prefix(':')?.trim();
    if type_detail.is_empty() {
        return None;
    }

    Some((name, optional, type_detail.to_string()))
}

fn parse_type_member_name(member: &str) -> Option<(String, usize)> {
    let mut chars = member.char_indices();
    let (_, first) = chars.next()?;
    if first == '"' || first == '\'' {
        for (idx, ch) in chars {
            if ch == first && !is_escaped(member, idx) {
                return Some((member[1..idx].to_string(), idx + ch.len_utf8()));
            }
        }
        return None;
    }

    let mut end = 0usize;
    for (idx, ch) in member.char_indices() {
        if idx == 0 {
            if !(ch == '_' || ch == '$' || ch.is_ascii_alphabetic()) {
                return None;
            }
        } else if !(ch == '_' || ch == '$' || ch.is_ascii_alphanumeric() || ch == '-') {
            break;
        }
        end = idx + ch.len_utf8();
    }

    (end > 0).then(|| (member[..end].to_string(), end))
}

fn strip_readonly(value: &str) -> &str {
    value
        .strip_prefix("readonly ")
        .map(str::trim_start)
        .unwrap_or(value)
}

pub(super) fn extract_balanced_after(
    source: &str,
    open_offset: usize,
    open: char,
    close: char,
) -> Option<(&str, usize)> {
    if !source[open_offset..].starts_with(open) {
        return None;
    }

    let mut depth = 0i32;
    let mut quote = None;
    let content_start = open_offset + open.len_utf8();
    let mut pos = open_offset;

    while pos < source.len() {
        let ch = source[pos..].chars().next()?;
        if let Some(open_quote) = quote {
            if ch == open_quote && !is_escaped(source, pos) {
                quote = None;
            }
            pos += ch.len_utf8();
            continue;
        }

        if ch == '"' || ch == '\'' || ch == '`' {
            quote = Some(ch);
        } else if ch == open {
            depth += 1;
        } else if ch == close && !(close == '>' && previous_non_ws_is(source, pos, '=')) {
            depth -= 1;
            if depth == 0 {
                return Some((&source[content_start..pos], pos + ch.len_utf8()));
            }
        }

        pos += ch.len_utf8();
    }

    None
}

pub(super) fn braced_body(value: &str) -> Option<&str> {
    let start = skip_ws(value, 0);
    if value.as_bytes().get(start) != Some(&b'{') {
        return None;
    }
    extract_balanced_after(value, start, '{', '}').map(|(body, _)| body)
}

pub(super) fn skip_ws(source: &str, mut pos: usize) -> usize {
    while pos < source.len() {
        let byte = source.as_bytes()[pos];
        if byte.is_ascii_whitespace() {
            pos += 1;
        } else {
            break;
        }
    }
    pos
}

fn is_escaped(source: &str, idx: usize) -> bool {
    let mut count = 0usize;
    let mut pos = idx;
    while pos > 0 && source.as_bytes()[pos - 1] == b'\\' {
        count += 1;
        pos -= 1;
    }
    count % 2 == 1
}

fn previous_non_ws_is(source: &str, idx: usize, expected: char) -> bool {
    source[..idx].chars().rev().find(|ch| !ch.is_whitespace()) == Some(expected)
}
