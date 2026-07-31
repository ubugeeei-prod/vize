//! Brace-aware folding for authored style blocks.

use tower_lsp::lsp_types::FoldingRange;
use vize_atelier_sfc::SfcStyleBlock;

use super::region;

#[derive(Clone, Copy)]
enum ScanState {
    Code,
    Quoted(u8),
    BlockComment,
    LineComment,
}

/// One region per multi-line brace block, in document order and O(n + r).
pub(super) fn style_regions(
    block: &SfcStyleBlock<'_>,
    content_start_line: u32,
) -> Vec<FoldingRange> {
    collect_style_regions(block, content_start_line).0
}

fn collect_style_regions(
    block: &SfcStyleBlock<'_>,
    content_start_line: u32,
) -> (Vec<FoldingRange>, usize) {
    if block.src.is_some() || block.content.is_empty() {
        return (Vec::new(), 0);
    }

    let bytes = block.content.as_bytes();
    let line_comments = matches!(
        block.lang.as_deref(),
        Some("less" | "scss" | "sass" | "stylus")
    );
    let mut state = ScanState::Code;
    let mut line = content_start_line;
    let mut stack = Vec::new();
    // Reserve a slot at `{` and fill it at `}` to preserve document order
    // without a final sort. Unterminated blocks leave an empty slot.
    let mut ranges = Vec::new();
    let mut cursor = 0;

    while cursor < bytes.len() {
        let byte = bytes[cursor];
        match state {
            ScanState::Code => {
                // A URL payload is opaque: `https://`, braces, and quotes in
                // it are data rather than style structure or line comments.
                if matches!(byte, b'u' | b'U')
                    && let Some(open_paren) = url_open_paren(bytes, cursor)
                {
                    advance_to(bytes, &mut cursor, open_paren + 1, &mut line);
                    skip_url(bytes, &mut cursor, &mut line);
                    continue;
                }

                match byte {
                    b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                        state = ScanState::BlockComment;
                        cursor += 2;
                    }
                    b'/' if line_comments && bytes.get(cursor + 1) == Some(&b'/') => {
                        state = ScanState::LineComment;
                        cursor += 2;
                    }
                    b'\'' | b'"' => {
                        state = ScanState::Quoted(byte);
                        cursor += 1;
                    }
                    b'\\' => skip_escape(bytes, &mut cursor, &mut line),
                    b'{' => {
                        let index = ranges.len();
                        ranges.push(None);
                        stack.push((line, index));
                        cursor += 1;
                    }
                    b'}' => {
                        if let Some((open_line, index)) = stack.pop() {
                            ranges[index] = region(open_line, line, None, None);
                        }
                        cursor += 1;
                    }
                    b'\n' => {
                        line += 1;
                        cursor += 1;
                    }
                    _ => cursor += 1,
                }
            }
            ScanState::Quoted(quote) => match byte {
                b'\\' => skip_escape(bytes, &mut cursor, &mut line),
                current if current == quote => {
                    state = ScanState::Code;
                    cursor += 1;
                }
                b'\n' => {
                    line += 1;
                    cursor += 1;
                }
                _ => cursor += 1,
            },
            ScanState::BlockComment => match byte {
                b'*' if bytes.get(cursor + 1) == Some(&b'/') => {
                    state = ScanState::Code;
                    cursor += 2;
                }
                b'\n' => {
                    line += 1;
                    cursor += 1;
                }
                _ => cursor += 1,
            },
            ScanState::LineComment => {
                if byte == b'\n' {
                    line += 1;
                    state = ScanState::Code;
                }
                cursor += 1;
            }
        }
    }

    let work = cursor + ranges.len();
    (ranges.into_iter().flatten().collect(), work)
}

#[cfg(test)]
pub(super) fn style_regions_with_metrics(
    block: &SfcStyleBlock<'_>,
    content_start_line: u32,
) -> (Vec<FoldingRange>, usize) {
    collect_style_regions(block, content_start_line)
}

fn url_open_paren(bytes: &[u8], cursor: usize) -> Option<usize> {
    if cursor > 0 && is_identifier_byte(bytes[cursor - 1]) {
        return None;
    }
    if !bytes.get(cursor..cursor + 3)?.eq_ignore_ascii_case(b"url") {
        return None;
    }
    if bytes
        .get(cursor + 3)
        .is_some_and(|byte| is_identifier_byte(*byte))
    {
        return None;
    }

    let mut open_paren = cursor + 3;
    while bytes
        .get(open_paren)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        open_paren += 1;
    }
    (bytes.get(open_paren) == Some(&b'(')).then_some(open_paren)
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'\\') || !byte.is_ascii()
}

fn skip_url(bytes: &[u8], cursor: &mut usize, line: &mut u32) {
    let mut quote = None;
    let mut depth = 1;
    while *cursor < bytes.len() {
        let byte = bytes[*cursor];
        match quote {
            Some(_) if byte == b'\\' => skip_escape(bytes, cursor, line),
            Some(open) if byte == open => {
                quote = None;
                *cursor += 1;
            }
            Some(_) if byte == b'\n' => {
                *line += 1;
                *cursor += 1;
            }
            Some(_) => *cursor += 1,
            None if matches!(byte, b'\'' | b'"') => {
                quote = Some(byte);
                *cursor += 1;
            }
            None if byte == b'\\' => skip_escape(bytes, cursor, line),
            None if byte == b'(' => {
                depth += 1;
                *cursor += 1;
            }
            None if byte == b')' => {
                depth -= 1;
                *cursor += 1;
                if depth == 0 {
                    break;
                }
            }
            None if byte == b'\n' => {
                *line += 1;
                *cursor += 1;
            }
            None => *cursor += 1,
        }
    }
}

fn advance_to(bytes: &[u8], cursor: &mut usize, end: usize, line: &mut u32) {
    *line += bytes[*cursor..end]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count() as u32;
    *cursor = end;
}

fn skip_escape(bytes: &[u8], cursor: &mut usize, line: &mut u32) {
    *cursor += 1;
    if bytes.get(*cursor) == Some(&b'\n') {
        *line += 1;
    }
    *cursor += usize::from(*cursor < bytes.len());
}
