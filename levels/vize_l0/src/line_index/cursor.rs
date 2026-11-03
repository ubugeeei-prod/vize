//! Allocation-free positions for offsets visited in source order.

use super::{Position, utf16::utf16_len};

/// Incremental UTF-16 coordinates for a sequence of byte offsets.
///
/// Nondecreasing queries never rescan a previous source segment. Backward
/// queries reset the cursor, retaining the same result as `offset_to_position`.
/// Offsets inside a UTF-8 character count that whole character, and offsets
/// beyond the source clamp to its end, just like the single-shot converter.
pub struct PositionCursor<'a> {
    source: &'a str,
    requested: usize,
    consumed: usize,
    position: Position,
    #[cfg(test)]
    scanned_segment_bytes: usize,
}

impl<'a> PositionCursor<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            requested: 0,
            consumed: 0,
            position: Position::default(),
            #[cfg(test)]
            scanned_segment_bytes: 0,
        }
    }

    pub fn position(&mut self, offset: usize) -> Position {
        let offset = offset.min(self.source.len());
        if offset < self.requested {
            self.consumed = 0;
            self.position = Position::default();
        }
        self.requested = offset;

        // A previous mid-character query has already counted this character.
        if offset <= self.consumed {
            return self.position;
        }
        let end = (offset..=self.source.len())
            .find(|at| self.source.is_char_boundary(*at))
            .unwrap_or(self.source.len());
        let text = self.source.get(self.consumed..end).unwrap_or_default();
        #[cfg(test)]
        {
            self.scanned_segment_bytes += text.len();
        }
        let mut line_start = 0;
        for at in memchr::memchr_iter(b'\n', text.as_bytes()) {
            self.position.line += 1;
            self.position.character = 0;
            line_start = at + 1;
        }
        self.position.character += utf16_len(text.get(line_start..).unwrap_or_default()) as u32;
        self.consumed = end;
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::PositionCursor;
    use crate::line_index::offset_to_position;

    #[test]
    fn ordered_and_backward_queries_match_at_every_byte_including_partial_characters() {
        for source in [
            "",
            "ascii",
            "\r\n\n\r",
            "é😀x\r\n日本\u{2028}z",
            "😀\n€\r\n🧭",
        ] {
            let mut cursor = PositionCursor::new(source);
            let end = source.len() + 4;
            for offsets in [
                (0..=end).collect::<Vec<_>>(),
                (0..=end).rev().collect(),
                (0..=end).flat_map(|at| [at, end, at, 0, at]).collect(),
            ] {
                for offset in offsets {
                    assert_eq!(
                        cursor.position(offset),
                        offset_to_position(source, offset),
                        "{source:?} at {offset}"
                    );
                }
            }
            assert_eq!(cursor.position(usize::MAX), offset_to_position(source, end));
        }
    }

    #[test]
    fn dense_unicode_and_sparse_multiline_endpoints_retain_complete_vectors() {
        for source in [
            "/* 日本 🧭 */ .x { color: red; }".repeat(64),
            "// 日本 🧭\r\n".repeat(128),
        ] {
            for stride in [1, 13, 4096] {
                let mut cursor = PositionCursor::new(&source);
                let offsets = (0..source.len()).step_by(stride);
                let actual: Vec<_> = offsets.clone().map(|at| cursor.position(at)).collect();
                let expected: Vec<_> = offsets.map(|at| offset_to_position(&source, at)).collect();
                assert_eq!(actual, expected);
                cursor.position(source.len());
                assert_eq!(cursor.scanned_segment_bytes, source.len());
            }
        }
    }
}
