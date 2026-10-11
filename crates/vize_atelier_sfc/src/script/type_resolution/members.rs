use super::{String, ToCompactString, split_top_level};

/// A single object member parsed out of a resolved type-literal body, e.g. the
/// `a?: string` in `{ a?: string; b: number }`.
pub(super) struct Member {
    pub(super) key: String,
    pub(super) optional: bool,
    pub(super) readonly: bool,
    pub(super) ty: String,
}

/// Parse a resolved object-literal member body (`a?: string; b: number`) into
/// individual members.
pub(super) fn parse_members(body: &str) -> Vec<Member> {
    // JSDoc/line comments would otherwise end up glued to member keys
    // (`/** ... */ side?`), making Pick/Omit key filtering match nothing.
    let body = strip_comments(body);
    let mut members = Vec::new();
    for part in split_top_level(&body, ';') {
        for sub in split_top_level(&part, ',') {
            // Members may also be newline-separated (no `;`/`,`), which would
            // otherwise glue every following member into the first one's type.
            for line in split_top_level(&sub, '\n') {
                if let Some(member) = parse_member(&line) {
                    members.push(member);
                }
            }
        }
    }
    members
}

/// Remove `/* ... */` and `// ...` comments from a type-literal body while
/// leaving string literal contents untouched.
fn strip_comments(body: &str) -> String {
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut keep_start = 0;
    let mut i = 0;
    while let Some(&byte) = bytes.get(i) {
        match byte {
            quote @ (b'\'' | b'"' | b'`') => {
                i += 1;
                while let Some(&c) = bytes.get(i)
                    && c != quote
                {
                    if c == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                out.push_str(body.get(keep_start..i).unwrap_or_default());
                let close = bytes
                    .get(i + 2..)
                    .and_then(|rest| rest.windows(2).position(|w| w == b"*/"))
                    .map_or(bytes.len(), |offset| i + 2 + offset + 2);
                i = close;
                keep_start = i;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                out.push_str(body.get(keep_start..i).unwrap_or_default());
                let eol = bytes
                    .get(i..)
                    .and_then(|rest| rest.iter().position(|&b| b == b'\n'))
                    .map_or(bytes.len(), |offset| i + offset);
                i = eol;
                keep_start = i;
            }
            _ => i += 1,
        }
    }
    out.push_str(body.get(keep_start..).unwrap_or_default());
    out
}

fn parse_member(part: &str) -> Option<Member> {
    let mut text = part.trim();
    if text.is_empty() {
        return None;
    }

    let mut readonly = false;
    if let Some(rest) = text.strip_prefix("readonly ") {
        readonly = true;
        text = rest.trim();
    }

    let colon = find_top_level_colon(text)?;
    let (key, ty) = text.split_at_checked(colon)?;
    let mut key = key.trim();
    let ty = ty.get(1..)?.trim();

    let optional = key.ends_with('?');
    if let Some(stripped) = key.strip_suffix('?') {
        key = stripped.trim();
    }
    if key.is_empty() || ty.is_empty() {
        return None;
    }

    Some(Member {
        key: key.to_compact_string(),
        optional,
        readonly,
        ty: ty.to_compact_string(),
    })
}

fn find_top_level_colon(text: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut prev = '\0';
    for (idx, ch) in text.char_indices() {
        match ch {
            '{' | '<' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            '>' => {
                if prev != '=' && depth > 0 {
                    depth -= 1;
                }
            }
            ':' if depth == 0 => return Some(idx),
            _ => {}
        }
        prev = ch;
    }
    None
}

/// Render members back into a `key: type; ...` body matching the resolver's
/// existing output shape.
pub(super) fn render_members(members: &[Member]) -> String {
    let mut out = String::default();
    for member in members {
        if !out.is_empty() {
            out.push_str("; ");
        }
        if member.readonly {
            out.push_str("readonly ");
        }
        out.push_str(&member.key);
        if member.optional {
            out.push('?');
        }
        out.push_str(": ");
        out.push_str(&member.ty);
    }
    out
}
