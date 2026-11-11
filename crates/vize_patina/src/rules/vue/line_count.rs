//! Line splitting for the whole-file and per-block line limits.
//!
//! A file that ends with a newline does not gain an extra empty line. That
//! matches ESLint `max-lines`. Per-block blank-line skipping uses the element
//! text, and a trailing newline inside that text stays an empty line.

pub(super) struct FileLine<'a> {
    pub start: u32,
    pub text: &'a str,
}

pub(super) fn file_lines(source: &str) -> Vec<FileLine<'_>> {
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (index, _) in source.match_indices('\n') {
        lines.push(FileLine {
            start: start as u32,
            text: trim_cr(source.get(start..index).unwrap_or("")),
        });
        start = index + 1;
    }
    if start < source.len() || lines.is_empty() {
        lines.push(FileLine {
            start: start as u32,
            text: trim_cr(source.get(start..).unwrap_or("")),
        });
    }
    lines
}

pub(super) fn empty_line_count(text: &str) -> usize {
    let mut count = 0usize;
    let mut start = 0usize;
    for (index, _) in text.match_indices('\n') {
        if trim_cr(text.get(start..index).unwrap_or(""))
            .trim()
            .is_empty()
        {
            count += 1;
        }
        start = index + 1;
    }
    if start < text.len() {
        if trim_cr(text.get(start..).unwrap_or("")).trim().is_empty() {
            count += 1;
        }
    } else if text.ends_with('\n') {
        count += 1;
    }
    count
}

fn trim_cr(text: &str) -> &str {
    text.strip_suffix('\r').unwrap_or(text)
}

fn at(bytes: &[u8], index: usize) -> Option<u8> {
    bytes.get(index).copied()
}

fn mark(mask: &mut [bool], index: usize) {
    if let Some(slot) = mask.get_mut(index) {
        *slot = true;
    }
}

/// Byte mask of `//`, `/* */`, and `<!-- -->` comments outside strings.
pub(super) fn comment_mask(source: &str) -> Vec<bool> {
    let bytes = source.as_bytes();
    let mut mask = vec![false; bytes.len()];
    let mut index = 0usize;
    while let Some(byte) = at(bytes, index) {
        if byte == b'"' || byte == b'\'' || byte == b'`' {
            index = skip_string(bytes, index);
            continue;
        }
        if byte == b'/' && at(bytes, index + 1) == Some(b'/') {
            while at(bytes, index).is_some_and(|byte| byte != b'\n') {
                mark(&mut mask, index);
                index += 1;
            }
            continue;
        }
        if byte == b'/' && at(bytes, index + 1) == Some(b'*') {
            mark(&mut mask, index);
            mark(&mut mask, index + 1);
            index += 2;
            while let Some(byte) = at(bytes, index) {
                mark(&mut mask, index);
                let closes = byte == b'/' && index > 0 && at(bytes, index - 1) == Some(b'*');
                index += 1;
                if closes {
                    break;
                }
            }
            continue;
        }
        if bytes.get(index..index + 4) == Some(b"<!--") {
            while let Some(byte) = at(bytes, index) {
                mark(&mut mask, index);
                let closes = byte == b'>'
                    && index >= 2
                    && at(bytes, index - 1) == Some(b'-')
                    && at(bytes, index - 2) == Some(b'-');
                index += 1;
                if closes {
                    break;
                }
            }
            continue;
        }
        index += 1;
    }
    mask
}

pub(super) fn comment_only(source: &str, line: &FileLine<'_>, mask: &[bool]) -> bool {
    let start = line.start as usize;
    let end = start + line.text.len();
    let mut saw_comment = false;
    let bytes = source.as_bytes().get(start..end).unwrap_or(&[]);
    for (offset, byte) in bytes.iter().copied().enumerate() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        if mask.get(start + offset) == Some(&true) {
            saw_comment = true;
            continue;
        }
        return false;
    }
    saw_comment
}

fn skip_string(bytes: &[u8], start: usize) -> usize {
    let Some(quote) = at(bytes, start) else {
        return bytes.len();
    };
    let mut index = start + 1;
    while let Some(byte) = at(bytes, index) {
        if byte == b'\\' {
            index = index.saturating_add(2).min(bytes.len());
            continue;
        }
        if byte == quote {
            return index + 1;
        }
        if byte == b'\n' && quote != b'`' {
            return index;
        }
        index += 1;
    }
    index
}
