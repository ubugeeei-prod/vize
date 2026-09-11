pub(super) fn has_nested_comment(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut depth: u32 = 0;
    let mut in_string: Option<u8> = None;
    let mut i = 0usize;

    while i < bytes.len() {
        let c = bytes[i];

        if let Some(quote) = in_string {
            if let Some(next) = consume_css_escape(bytes, i) {
                i = next;
                continue;
            }
            if c == quote {
                in_string = None;
            }
            i += 1;
            continue;
        }

        if let Some(next) = consume_css_escape(bytes, i) {
            i = next;
            continue;
        }

        if let Some(next) = consume_unquoted_url(bytes, i) {
            i = next;
            continue;
        }

        match c {
            b'"' | b'\'' => {
                in_string = Some(c);
                i += 1;
            }
            b'{' => {
                depth = depth.saturating_add(1);
                i += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                if depth > 0 {
                    return true;
                }
                i = find_comment_end(bytes, i + 2);
            }
            _ => i += 1,
        }
    }

    false
}

#[derive(Clone, Copy)]
pub(super) enum SegmentKind {
    Code,
    Comment,
}

pub(super) struct CssSegment<'a> {
    pub(super) kind: SegmentKind,
    pub(super) content: &'a str,
}

/// Split CSS source into alternating code and top-level comment segments.
pub(super) fn split_top_level_comments(source: &str) -> Vec<CssSegment<'_>> {
    let bytes = source.as_bytes();
    let mut segments: Vec<CssSegment<'_>> = Vec::new();
    let mut depth: u32 = 0;
    let mut in_string: Option<u8> = None;
    let mut last_split = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        let c = bytes[i];

        if let Some(quote) = in_string {
            if let Some(next) = consume_css_escape(bytes, i) {
                i = next;
                continue;
            }
            if c == quote {
                in_string = None;
            }
            i += 1;
            continue;
        }

        if let Some(next) = consume_css_escape(bytes, i) {
            i = next;
            continue;
        }

        if let Some(next) = consume_unquoted_url(bytes, i) {
            i = next;
            continue;
        }

        match c {
            b'"' | b'\'' => {
                in_string = Some(c);
                i += 1;
            }
            b'{' => {
                depth = depth.saturating_add(1);
                i += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                let comment_end = find_comment_end(bytes, i + 2);
                if depth == 0 {
                    if i > last_split {
                        segments.push(CssSegment {
                            kind: SegmentKind::Code,
                            content: &source[last_split..i],
                        });
                    }
                    segments.push(CssSegment {
                        kind: SegmentKind::Comment,
                        content: &source[i..comment_end],
                    });
                    last_split = comment_end;
                }
                i = comment_end;
            }
            _ => i += 1,
        }
    }

    if last_split < bytes.len() {
        segments.push(CssSegment {
            kind: SegmentKind::Code,
            content: &source[last_split..],
        });
    }

    segments
}

fn consume_css_escape(bytes: &[u8], from: usize) -> Option<usize> {
    if bytes.get(from) != Some(&b'\\') {
        return None;
    }

    let mut i = from + 1;
    if i >= bytes.len() {
        return Some(i);
    }

    if bytes[i].is_ascii_hexdigit() {
        let mut digit_count = 0u8;
        while i < bytes.len() && digit_count < 6 && bytes[i].is_ascii_hexdigit() {
            i += 1;
            digit_count += 1;
        }

        if i < bytes.len() && is_css_whitespace(bytes[i]) {
            if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                i += 2;
            } else {
                i += 1;
            }
        }

        return Some(i);
    }

    if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
        return Some(i + 2);
    }

    Some(i + 1)
}

fn consume_unquoted_url(bytes: &[u8], from: usize) -> Option<usize> {
    if from > 0 && is_css_ident_byte(bytes[from - 1]) {
        return None;
    }
    if !starts_with_ascii_case_insensitive(bytes, from, b"url") {
        return None;
    }

    let mut i = from + 3;
    while i < bytes.len() && is_css_whitespace(bytes[i]) {
        i += 1;
    }
    if bytes.get(i) != Some(&b'(') {
        return None;
    }
    i += 1;
    while i < bytes.len() && is_css_whitespace(bytes[i]) {
        i += 1;
    }
    if matches!(bytes.get(i), Some(b'"' | b'\'')) {
        return None;
    }

    while i < bytes.len() {
        if let Some(next) = consume_css_escape(bytes, i) {
            i = next;
            continue;
        }
        if bytes[i] == b')' {
            return Some(i + 1);
        }
        i += 1;
    }

    Some(i)
}

fn starts_with_ascii_case_insensitive(bytes: &[u8], from: usize, needle: &[u8]) -> bool {
    bytes
        .get(from..from + needle.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(needle))
}

fn is_css_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

fn is_css_whitespace(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\r' | b'\x0c' | b' ')
}

fn find_comment_end(bytes: &[u8], from: usize) -> usize {
    let mut j = from;
    while j + 1 < bytes.len() {
        if bytes[j] == b'*' && bytes[j + 1] == b'/' {
            return j + 2;
        }
        j += 1;
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::{SegmentKind, has_nested_comment, split_top_level_comments};

    #[test]
    fn css_escapes_do_not_affect_comment_depth() {
        let escaped_simple = ".escaped\\{name { color: red; }\n/* top-level */";
        let escaped_hex = ".escaped\\00007b name { color: red; }\n/* top-level */";
        let escaped_hex_newline = ".escaped\\00007b\nname { color: red; }\n/* top-level */";
        let escaped_close = ".rule { color: \\}; /* nested */ }\n/* top-level */";
        let declaration_url = ".asset { background: url(https://example.test/a/*/icon.svg); }";
        let import_url = "@import url(https://example.test/a/*/reset.css);\n/* top-level */";
        let unescaped = ".unescaped { color: red; /* nested */ }";

        assert_eq!(has_nested_comment(escaped_simple), false);
        assert_eq!(has_nested_comment(escaped_hex), false);
        assert_eq!(has_nested_comment(escaped_hex_newline), false);
        assert_eq!(has_nested_comment(escaped_close), true);
        assert_eq!(has_nested_comment(declaration_url), false);
        assert_eq!(has_nested_comment(import_url), false);
        assert_eq!(has_nested_comment(unescaped), true);

        assert_eq!(top_level_comment_count(escaped_hex), 1);
        assert_eq!(top_level_comment_count(escaped_hex_newline), 1);
        assert_eq!(top_level_comment_count(escaped_close), 1);
        assert_eq!(top_level_comment_count(import_url), 1);
    }

    fn top_level_comment_count(source: &str) -> usize {
        split_top_level_comments(source)
            .iter()
            .filter(|segment| matches!(segment.kind, SegmentKind::Comment))
            .count()
    }
}
