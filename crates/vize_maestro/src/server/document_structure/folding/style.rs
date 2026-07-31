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

pub(super) fn style_regions(block: &SfcStyleBlock<'_>) -> Vec<FoldingRange> {
    if block.src.is_some() || block.content.is_empty() {
        return Vec::new();
    }

    let source = block.content.as_ref();
    let bytes = source.as_bytes();
    let line_comments = matches!(
        block.lang.as_deref(),
        Some("less" | "scss" | "sass" | "stylus")
    );
    let mut state = ScanState::Code;
    let mut line = block.loc.start_line.saturating_sub(1) as u32;
    let mut stack = Vec::new();
    let mut ranges = Vec::new();
    let mut cursor = 0;

    while cursor < bytes.len() {
        let byte = bytes[cursor];
        match state {
            ScanState::Code => match byte {
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
                    stack.push(line);
                    cursor += 1;
                }
                b'}' => {
                    if let Some(open_line) = stack.pop()
                        && let Some(range) = region(open_line, line, None, None)
                    {
                        ranges.push(range);
                    }
                    cursor += 1;
                }
                b'\n' => {
                    line += 1;
                    cursor += 1;
                }
                _ => cursor += 1,
            },
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

    ranges.sort_by_key(|range| (range.start_line, range.end_line));
    ranges
}

fn skip_escape(bytes: &[u8], cursor: &mut usize, line: &mut u32) {
    *cursor += 1;
    if bytes.get(*cursor) == Some(&b'\n') {
        *line += 1;
    }
    *cursor += usize::from(*cursor < bytes.len());
}
