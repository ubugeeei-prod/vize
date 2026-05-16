//! Position and range utilities for converting between LSP and internal representations.

use ropey::Rope;
use tower_lsp::lsp_types::{Position, Range};

/// Convert a byte offset to an LSP Position (0-based line and character).
pub fn offset_to_position(rope: &Rope, offset: usize) -> Option<Position> {
    if offset > rope.len_bytes() {
        return None;
    }

    // Find the line containing this offset
    let char_idx = rope.try_byte_to_char(offset).ok()?;
    let line = rope.char_to_line(char_idx);
    let line_start_char = rope.line_to_char(line);
    let character = char_idx - line_start_char;

    Some(Position {
        line: line as u32,
        character: character as u32,
    })
}

/// Convert an LSP Position (0-based) to a byte offset.
pub fn position_to_offset(rope: &Rope, position: Position) -> Option<usize> {
    let line = position.line as usize;
    let character = position.character as usize;

    if line >= rope.len_lines() {
        return None;
    }

    let line_start_char = rope.line_to_char(line);
    let line_len = rope.line(line).len_chars();

    // Clamp character to line length
    let char_in_line = character.min(line_len);
    let char_idx = line_start_char + char_in_line;

    rope.try_char_to_byte(char_idx).ok()
}

/// Convert internal 1-based Position to LSP 0-based Position.
pub fn internal_to_lsp_position(pos: &vize_relief::Position) -> Position {
    Position {
        line: pos.line.saturating_sub(1),
        character: pos.column.saturating_sub(1),
    }
}

/// Convert internal SourceLocation to LSP Range.
pub fn source_location_to_range(loc: &vize_relief::SourceLocation) -> Range {
    Range {
        start: internal_to_lsp_position(&loc.start),
        end: internal_to_lsp_position(&loc.end),
    }
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
    let mut current_line = 0u32;
    let mut current_offset = 0usize;

    for (i, ch) in content.char_indices() {
        if current_line == line {
            // We're on the target line, count characters
            let line_start = current_offset;

            for (char_count, (j, c)) in content[line_start..].char_indices().enumerate() {
                if c == '\n' || char_count as u32 == character {
                    return line_start + j;
                }
            }
            // End of file reached
            return content.len();
        }

        if ch == '\n' {
            current_line += 1;
        }
        current_offset = i + ch.len_utf8();
    }

    // If we're past all lines, return end of content
    content.len()
}

/// Get the range of a line (0-based line number).
pub fn line_range(rope: &Rope, line: usize) -> Option<Range> {
    if line >= rope.len_lines() {
        return None;
    }

    let line_text = rope.line(line);
    let line_len = line_text.len_chars();

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
mod tests {
    use super::{internal_to_lsp_position, offset_to_position, position_to_offset};
    use ropey::Rope;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn test_offset_to_position() {
        let rope = Rope::from_str("hello\nworld\n");

        // Start of file
        assert_eq!(
            offset_to_position(&rope, 0),
            Some(Position {
                line: 0,
                character: 0,
            })
        );

        // Middle of first line
        assert_eq!(
            offset_to_position(&rope, 3),
            Some(Position {
                line: 0,
                character: 3,
            })
        );

        // Start of second line
        assert_eq!(
            offset_to_position(&rope, 6),
            Some(Position {
                line: 1,
                character: 0,
            })
        );

        // End of file
        assert_eq!(
            offset_to_position(&rope, 12),
            Some(Position {
                line: 2,
                character: 0,
            })
        );
    }

    #[test]
    fn test_position_to_offset() {
        let rope = Rope::from_str("hello\nworld\n");

        // Start of file
        assert_eq!(
            position_to_offset(
                &rope,
                Position {
                    line: 0,
                    character: 0,
                }
            ),
            Some(0)
        );

        // Middle of first line
        assert_eq!(
            position_to_offset(
                &rope,
                Position {
                    line: 0,
                    character: 3,
                }
            ),
            Some(3)
        );

        // Start of second line
        assert_eq!(
            position_to_offset(
                &rope,
                Position {
                    line: 1,
                    character: 0,
                }
            ),
            Some(6)
        );
    }

    #[test]
    fn test_internal_to_lsp_position() {
        let internal = vize_relief::Position {
            offset: 10,
            line: 2,
            column: 5,
        };

        let lsp = internal_to_lsp_position(&internal);
        assert_eq!(lsp.line, 1);
        assert_eq!(lsp.character, 4);
    }
}
