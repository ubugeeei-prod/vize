//! Byte-offset to (line, UTF-16 column) mapping for LSP coordinates.
//!
//! LSP `Position.character` is measured in **UTF-16 code units**, not bytes and
//! not Unicode scalar values. Astral (non-BMP) characters such as emoji encode
//! as a UTF-16 surrogate pair (`char::len_utf16() == 2`) and therefore advance
//! the column by two, matching what `vue-tsc` / `@vue/language-tools` report.
//!
//! This module is the single home for that column math. Several diagnostic and
//! semantic-token collectors across `vize_canon` and `vize_maestro` used to
//! hand-roll it; at least one copy counted **bytes** instead of UTF-16 code
//! units (the #1223 class of bug). Centralizing it here keeps the conversion
//! correct in one place. See issue #1389.

use crate::lsp::Position;

mod line_breaks;
mod utf16;
pub use line_breaks::LineBreaks;
pub use utf16::{utf16_len, utf16_offset};

/// Precomputed byte offsets of every line start in a source string.
///
/// Building this once and reusing it turns the per-call `O(offset)` scan of
/// [`offset_to_line_col`] into a binary search over line starts plus a short
/// UTF-16 column scan within the target line. Callers that map many offsets
/// against the same `content` (diagnostics collectors, semantic-token
/// collectors) build it once and thread it through.
pub struct LineIndex<'a> {
    source: &'a str,
    /// Byte offset of the start of each line. `line_starts[0]` is always `0`.
    line_starts: crate::SmallVec<[usize; 8]>,
}

impl<'a> LineIndex<'a> {
    /// Count line breaks before filling the index to avoid excess capacity or growth.
    pub fn new(source: &'a str) -> Self {
        let breaks = memchr::memchr_iter(b'\n', source.as_bytes());
        let mut line_starts = crate::SmallVec::with_capacity(breaks.clone().count() + 1);
        line_starts.push(0);
        line_starts.extend(breaks.map(|at| at + 1));
        Self {
            source,
            line_starts,
        }
    }

    /// Convert a byte offset to `(line, column)`, both 0-indexed for LSP.
    ///
    /// `column` is measured in UTF-16 code units. Offsets past the end of the
    /// source are clamped. An offset that falls mid-character counts the partial
    /// character (so it never panics), matching the legacy break-at-top scans.
    ///
    /// `\r\n` is handled naturally: the `\r` is an ordinary character preceding
    /// the `\n`, so the column at end-of-line includes it and the next line
    /// starts immediately after the `\n`.
    pub fn line_col(&self, offset: usize) -> (u32, u32) {
        let offset = offset.min(self.source.len());

        // Greatest line start <= offset.
        // `line_starts` begins with 0, so a predecessor always exists.
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(next) => next.saturating_sub(1),
        };
        let line_start = self.line_starts.get(line).copied().unwrap_or(0);

        // Sum UTF-16 code units of every character on this line whose byte
        // start is before `offset`.
        let line_text = self.source.get(line_start..).unwrap_or_default();
        let col = utf16::prefix_len(line_text, offset.saturating_sub(line_start));
        (line as u32, col as u32)
    }

    /// Convert a byte offset to an LSP [`Position`].
    pub fn position(&self, offset: usize) -> Position {
        let (line, character) = self.line_col(offset);
        Position { line, character }
    }

    /// Convert an LSP `(line, UTF-16 column)` position to a byte offset.
    pub fn line_col_to_offset(&self, line: u32, column: u32) -> Option<usize> {
        let line = usize::try_from(line).ok()?;
        let start = *self.line_starts.get(line)?;
        let end = self
            .line_starts
            .get(line + 1)
            .map(|next| next.saturating_sub(1))
            .unwrap_or(self.source.len());
        utf16_offset(self.source.get(start..end)?, column).map(|offset| start + offset)
    }
}

/// Convert a byte offset to `(line, column)`, both 0-indexed for LSP, with
/// `column` in UTF-16 code units.
///
/// Single-shot conversion without allocating an index. Hot paths that map many
/// offsets against the same content should build a [`LineIndex`] once and call
/// [`LineIndex::line_col`] instead.
pub fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let offset = offset.min(source.len());
    let mut line = 0;
    let mut start = 0;
    let head = source.as_bytes().get(..offset).unwrap_or_default();
    for at in memchr::memchr_iter(b'\n', head) {
        line += 1;
        start = at + 1;
    }
    let tail = source.get(start..).unwrap_or_default();
    (line, utf16::prefix_len(tail, offset - start) as u32)
}

/// Convert a byte offset to an LSP [`Position`] (single-shot convenience).
pub fn offset_to_position(source: &str, offset: usize) -> Position {
    let (line, character) = offset_to_line_col(source, offset);
    Position { line, character }
}

#[cfg(test)]
mod tests {
    use super::{LineIndex, offset_to_line_col};

    #[test]
    fn indexed_and_allocation_free_positions_match_at_every_byte() {
        for source in ["", "a\r\nb\n", "é😀x\n€\u{2028}z", "\n\n\r"] {
            let index = LineIndex::new(source);
            for offset in 0..=source.len() + 2 {
                let mut expected = (0, 0);
                for (at, ch) in source.char_indices() {
                    if at >= offset {
                        break;
                    }
                    if ch == '\n' {
                        expected = (expected.0 + 1, 0);
                    } else {
                        expected.1 += ch.len_utf16() as u32;
                    }
                }
                assert_eq!(index.line_col(offset), expected, "{source:?} at {offset}");
                assert_eq!(offset_to_line_col(source, offset), expected);
            }
        }
    }

    #[test]
    fn zero_indexed_for_lsp() {
        assert_eq!(offset_to_line_col("one\ntwo", 0), (0, 0));
        assert_eq!(offset_to_line_col("one\ntwo", 3), (0, 3));
        assert_eq!(offset_to_line_col("one\ntwo", 4), (1, 0));
        assert_eq!(offset_to_line_col("one\ntwo", 6), (1, 2));
    }

    #[test]
    fn clamps_offset_past_end() {
        assert_eq!(offset_to_line_col("ab", 99), (0, 2));
    }

    #[test]
    fn counts_utf16_code_units_for_non_bmp_char() {
        // U+1F600 GRINNING FACE: 4 UTF-8 bytes, 2 UTF-16 code units (surrogate
        // pair). The column at the byte just after it must be 2, not 1 (chars)
        // and not 4 (bytes).
        let source = "😀x";
        let emoji_bytes = '😀'.len_utf8();
        assert_eq!(emoji_bytes, 4);
        // Just after the emoji: 2 UTF-16 code units.
        assert_eq!(offset_to_line_col(source, emoji_bytes), (0, 2));
        // Just after the trailing 'x': 3 UTF-16 code units.
        assert_eq!(offset_to_line_col(source, emoji_bytes + 1), (0, 3));
    }

    #[test]
    fn converts_utf16_positions_back_to_byte_offsets() {
        let source = "plain\n😀x";
        let index = LineIndex::new(source);

        assert_eq!(index.line_col_to_offset(1, 0), Some(6));
        assert_eq!(index.line_col_to_offset(1, 2), Some(10));
        assert_eq!(index.line_col_to_offset(1, 3), Some(11));
        assert_eq!(index.line_col_to_offset(1, 1), None);
        assert_eq!(index.line_col_to_offset(2, 0), None);
    }

    #[test]
    fn non_bmp_char_after_text() {
        let source = "const icon = \"😀\"; missing";
        let offset = source.find("missing").unwrap();
        // 13 chars `const icon = ` + `"` (1) + emoji (2 UTF-16 units) + `"; ` (3)
        // = 13 + 1 + 2 + 3 = 19.
        assert_eq!(offset_to_line_col(source, offset), (0, 19));
    }

    #[test]
    fn crlf_line_endings() {
        let source = "ab\r\ncd";
        // End of first line, on the '\r'.
        assert_eq!(offset_to_line_col(source, 2), (0, 2));
        // On the '\n' itself: still line 0, column counts the '\r' too.
        assert_eq!(offset_to_line_col(source, 3), (0, 3));
        // Start of the second line, immediately after '\n'.
        assert_eq!(offset_to_line_col(source, 4), (1, 0));
        // 'd'.
        assert_eq!(offset_to_line_col(source, 5), (1, 1));
    }

    #[test]
    fn crlf_with_non_bmp_char() {
        let source = "😀\r\n😀";
        let first = '😀'.len_utf8(); // 4
        // After the first emoji, before '\r'.
        assert_eq!(offset_to_line_col(source, first), (0, 2));
        // Start of second line.
        assert_eq!(offset_to_line_col(source, first + 2), (1, 0));
        // After the second emoji.
        assert_eq!(offset_to_line_col(source, first + 2 + first), (1, 2));
    }

    #[test]
    fn line_index_matches_single_shot() {
        let source = "alpha\nβγ😀δ\r\nlast line";
        let index = LineIndex::new(source);
        for offset in 0..=source.len() {
            // Skip offsets that fall inside a multi-byte char boundary for the
            // single-shot helper's char_indices alignment; both implementations
            // share the same break-at-top semantics, so compare directly.
            assert_eq!(
                index.line_col(offset),
                offset_to_line_col(source, offset),
                "mismatch at offset {offset}"
            );
        }
    }
}
