//! Lightweight same-file ecosystem context detection.

#[derive(Debug, Clone, Copy)]
pub(super) struct StringLiteralContext<'a> {
    pub(super) before_open: &'a str,
    pub(super) open: usize,
}

pub(super) fn string_literal_at_cursor(
    content: &str,
    offset: usize,
) -> Option<StringLiteralContext<'_>> {
    if content.is_empty() {
        return None;
    }

    let offset = offset.min(content.len());
    let bytes = content.as_bytes();
    let mut pos = offset;

    while pos > 0 {
        pos -= 1;
        let byte = bytes[pos];
        if byte == b'\n' || byte == b'\r' {
            return None;
        }
        if (byte == b'\'' || byte == b'"') && !is_escaped(bytes, pos) {
            if !cursor_is_before_closing_quote(bytes, pos + 1, offset, byte) {
                return None;
            }
            return Some(StringLiteralContext {
                before_open: &content[..pos],
                open: pos,
            });
        }
    }

    None
}

pub(super) fn preceding_property_is_name(before_open: &str) -> bool {
    let before_colon = before_open.trim_end();
    let Some(before_colon) = before_colon.strip_suffix(':') else {
        return false;
    };
    let before_name = before_colon.trim_end();
    let Some(prefix) = before_name.strip_suffix("name") else {
        return false;
    };
    prefix
        .as_bytes()
        .last()
        .map(|byte| !is_ident_byte(*byte))
        .unwrap_or(true)
}

pub(super) fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn is_escaped(bytes: &[u8], quote: usize) -> bool {
    let mut slash_count = 0usize;
    let mut pos = quote;
    while pos > 0 && bytes[pos - 1] == b'\\' {
        slash_count += 1;
        pos -= 1;
    }
    slash_count % 2 == 1
}

fn cursor_is_before_closing_quote(bytes: &[u8], mut pos: usize, cursor: usize, quote: u8) -> bool {
    while pos < bytes.len() {
        let byte = bytes[pos];
        if byte == b'\n' || byte == b'\r' {
            return false;
        }
        if byte == quote && !is_escaped(bytes, pos) {
            return cursor <= pos;
        }
        pos += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{preceding_property_is_name, string_literal_at_cursor};

    #[test]
    fn detects_string_literal_before_cursor() {
        let source = "router.push({ name: \"home\" })";
        let offset = source.find("ho").unwrap() + 2;
        let ctx = string_literal_at_cursor(source, offset).unwrap();

        assert_eq!(ctx.open, source.find('"').unwrap());
        assert!(preceding_property_is_name(ctx.before_open));
    }

    #[test]
    fn ignores_content_outside_string_literals() {
        assert!(string_literal_at_cursor("router.push({ name: home })", 22).is_none());

        let source = "router.push({ name: \"home\" })";
        let after_close = source.find("\" })").unwrap() + 1;
        assert!(string_literal_at_cursor(source, after_close + 1).is_none());
    }
}
