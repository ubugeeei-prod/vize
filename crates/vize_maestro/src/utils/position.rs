//! Position and range utilities for converting between LSP and internal representations.

use crate::document::DocumentText;
use ropey::RopeSlice;
use tower_lsp::lsp_types::{Position, Range};

/// Convert a byte offset to an LSP Position (0-based line and character).
pub fn offset_to_position(rope: &DocumentText, offset: usize) -> Option<Position> {
    if offset > rope.len_bytes() {
        return None;
    }

    // Find the line containing this offset
    let line = rope.byte_to_line(offset);
    let line_offset = offset - rope.line_to_byte(line);
    if line_offset == 0 {
        return Some(Position::new(line as u32, 0));
    }
    let text = rope.line(line);
    let char_idx = text.byte_to_char(line_offset);
    // Rope nodes already store UTF-16 lengths. Use those measures instead of
    // walking the whole line for every edit, diagnostic, or token.
    let character = text.char_to_utf16_cu(char_idx);

    Some(Position {
        line: line as u32,
        character: character as u32,
    })
}

/// Convert an LSP Position (0-based) to a byte offset.
pub fn position_to_offset(rope: &DocumentText, position: Position) -> Option<usize> {
    let line = position.line as usize;
    let character = position.character as usize;

    if line >= rope.len_lines() {
        return None;
    }
    if character == 0 {
        return Some(rope.line_to_byte(line));
    }

    let text = rope.line(line);
    let line_end_char = line_content_end(text);
    let char_idx = text.try_utf16_cu_to_char(character).ok()?;
    // Rope rounds a surrogate-pair interior down. LSP edit boundaries must
    // instead reject it, and must never spill into the next line or CRLF.
    if char_idx > line_end_char || text.char_to_utf16_cu(char_idx) != character {
        return None;
    }
    Some(rope.line_to_byte(line) + text.char_to_byte(char_idx))
}

fn line_content_end(text: RopeSlice<'_>) -> usize {
    let mut end = text.len_chars();
    if end > 0 && text.char(end - 1) == '\n' {
        end -= 1;
    }
    if end > 0 && text.char(end - 1) == '\r' {
        end -= 1;
    }
    end
}

/// Convert a byte offset in a string to an LSP position.
///
/// LSP `character` values are UTF-16 code units, not Rust scalar-value counts.
/// This helper mirrors [`offset_to_position`] for call sites that already have
/// string content instead of a reusable rope.
pub fn offset_to_position_str(content: &str, offset: usize) -> Position {
    let (line, character) =
        vize_s0::line_index::LineBreaks::Lsp.offset_to_position(content, offset);
    Position { line, character }
}

/// Create an LSP Range from start and end positions.
pub fn make_range(start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> Range {
    Range {
        start: Position {
            line: start_line,
            character: start_char,
        },
        end: Position {
            line: end_line,
            character: end_char,
        },
    }
}

/// Convert LSP position (0-based line/character) to byte offset in a string.
///
/// This is a convenience function that works directly with string content.
/// For better performance with repeated conversions, use the Rope-based version.
#[inline]
pub fn position_to_offset_str(content: &str, line: u32, character: u32) -> usize {
    let start = vize_s0::line_index::LineBreaks::Lsp
        .line_starts(content)
        .nth(line as usize)
        .unwrap_or(content.len());
    let mut utf16_units = 0u32;
    for (at, ch) in content[start..].char_indices() {
        if matches!(ch, '\r' | '\n') || utf16_units >= character {
            return start + at;
        }
        utf16_units += ch.len_utf16() as u32;
    }
    content.len()
}

/// Get the range of a line (0-based line number).
pub fn line_range(rope: &DocumentText, line: usize) -> Option<Range> {
    if line >= rope.len_lines() {
        return None;
    }

    let text = rope.line(line);
    let line_len = text.char_to_utf16_cu(line_content_end(text));

    Some(Range {
        start: Position {
            line: line as u32,
            character: 0,
        },
        end: Position {
            line: line as u32,
            character: line_len as u32,
        },
    })
}

#[cfg(test)]
mod tests;
