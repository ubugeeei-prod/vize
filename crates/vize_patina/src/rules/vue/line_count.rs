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
            text: trim_cr(&source[start..index]),
        });
        start = index + 1;
    }
    if start < source.len() || lines.is_empty() {
        lines.push(FileLine {
            start: start as u32,
            text: trim_cr(&source[start..]),
        });
    }
    lines
}

pub(super) fn empty_line_count(text: &str) -> usize {
    let mut count = 0usize;
    let mut start = 0usize;
    for (index, _) in text.match_indices('\n') {
        if trim_cr(&text[start..index]).trim().is_empty() {
            count += 1;
        }
        start = index + 1;
    }
    if start < text.len() {
        if trim_cr(&text[start..]).trim().is_empty() {
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

/// Byte mask of `//`, `/* */`, and `<!-- -->` comments outside strings.
pub(super) fn comment_mask(source: &str) -> Vec<bool> {
    let bytes = source.as_bytes();
    let mut mask = vec![false; bytes.len()];
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'"' || byte == b'\'' || byte == b'`' {
            index = skip_string(bytes, index);
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'/') {
            while index < bytes.len() && bytes[index] != b'\n' {
                mask[index] = true;
                index += 1;
            }
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            mask[index] = true;
            mask[index + 1] = true;
            index += 2;
            while index < bytes.len() {
                mask[index] = true;
                if bytes[index] == b'/' && index > 0 && bytes[index - 1] == b'*' {
                    index += 1;
                    break;
                }
                index += 1;
            }
            continue;
        }
        if bytes.get(index..index + 4) == Some(b"<!--") {
            while index < bytes.len() {
                mask[index] = true;
                if bytes[index] == b'>'
                    && index >= 2
                    && bytes[index - 1] == b'-'
                    && bytes[index - 2] == b'-'
                {
                    index += 1;
                    break;
                }
                index += 1;
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
    for (offset, byte) in source.as_bytes()[start..end].iter().copied().enumerate() {
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
    let quote = bytes[start];
    let mut index = start + 1;
    while index < bytes.len() {
        let byte = bytes[index];
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
