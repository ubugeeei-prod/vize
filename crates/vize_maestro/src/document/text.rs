//! LSP line boundaries over Ropey's Unicode line metrics.

use std::{fmt, ops::Range};

use ropey::{Rope, RopeSlice};
use vize_l0::SmallVec;

/// Editable document text. Only CR, LF and CRLF create LSP lines.
///
/// Ropey's Unicode line metrics also count VT, FF, NEL, LS and PS. Keep only
/// those exceptional boundaries alongside the text, so normal documents need
/// no additional heap allocation or per-request scan. The rope is private and
/// replacements update both together; no independently invalidated cache exists.
#[derive(Clone, Debug)]
pub struct DocumentText {
    rope: Rope,
    ignored_breaks: SmallVec<[IgnoredBreak; 1]>,
}

#[derive(Clone, Copy, Debug)]
struct IgnoredBreak {
    /// UTF-8 byte immediately after the character Ropey counts as a newline.
    after: usize,
    /// LSP line containing this character (several may share one LSP line).
    line: usize,
}

impl DocumentText {
    pub fn new(source: &str) -> Self {
        let rope = Rope::from_str(source);
        // A single Ropey line has no exceptional breaks. Its byte/character
        // counts also let ASCII documents skip the Unicode candidate search.
        let ignored_breaks = if rope.len_lines() == 1 {
            SmallVec::new()
        } else {
            ignored_break_ends(source, rope.len_bytes() != rope.len_chars())
                .map(|after| IgnoredBreak { after, line: 0 })
                .collect()
        };
        let mut text = Self {
            rope,
            ignored_breaks,
        };
        text.refresh_ignored_lines();
        text
    }

    #[inline]
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    #[inline]
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines() - self.ignored_breaks.len()
    }

    pub fn byte_to_line(&self, byte: usize) -> usize {
        self.rope.byte_to_line(byte)
            - self
                .ignored_breaks
                .partition_point(|entry| entry.after <= byte)
    }

    pub fn line_to_byte(&self, line: usize) -> usize {
        if self.ignored_breaks.is_empty() {
            return self.rope.line_to_byte(line);
        }
        assert!(line <= self.len_lines(), "LSP line index out of bounds");
        let skipped = self
            .ignored_breaks
            .partition_point(|entry| entry.line < line);
        self.rope.line_to_byte(line + skipped)
    }

    pub fn line(&self, line: usize) -> RopeSlice<'_> {
        if self.ignored_breaks.is_empty() {
            return self.rope.line(line);
        }
        assert!(line < self.len_lines(), "LSP line index out of bounds");
        self.rope
            .byte_slice(self.line_to_byte(line)..self.line_to_byte(line + 1))
    }

    #[inline]
    pub fn try_byte_to_char(&self, byte: usize) -> Result<usize, ropey::Error> {
        self.rope.try_byte_to_char(byte)
    }

    /// Replace a character range and update exceptional boundaries atomically.
    pub fn replace(&mut self, range: Range<usize>, inserted: &str) {
        assert!(range.start <= range.end, "reversed document edit");
        let start = self.rope.char_to_byte(range.start);
        let end = self.rope.char_to_byte(range.end);
        let first = self
            .ignored_breaks
            .partition_point(|entry| entry.after <= start);
        let last = self
            .ignored_breaks
            .partition_point(|entry| entry.after <= end);
        self.ignored_breaks.drain(first..last);
        for entry in self.ignored_breaks.iter_mut().skip(first) {
            entry.after = entry.after - end + start + inserted.len();
        }
        self.ignored_breaks.insert_many(
            first,
            ignored_break_ends(inserted, true).map(|after| IgnoredBreak {
                after: start + after,
                line: 0,
            }),
        );
        let start_char = range.start;
        self.rope.remove(range);
        self.rope.insert(start_char, inserted);
        // Recalculate from Ropey's updated tree, including CRLF pairs that an
        // edit split or joined. Only exceptional characters need this lookup.
        self.refresh_ignored_lines();
    }

    fn refresh_ignored_lines(&mut self) {
        for (index, entry) in self.ignored_breaks.iter_mut().enumerate() {
            entry.line = self.rope.byte_to_line(entry.after) - index - 1;
        }
    }
}

impl fmt::Display for DocumentText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.rope, formatter)
    }
}

impl PartialEq<&str> for DocumentText {
    fn eq(&self, other: &&str) -> bool {
        self.rope == *other
    }
}

fn ignored_break_ends(source: &str, scan_unicode: bool) -> impl Iterator<Item = usize> + '_ {
    let bytes = source.as_bytes();
    let mut ascii = memchr::memchr2_iter(0x0b, 0x0c, bytes)
        .map(|at| at + 1)
        .peekable();
    let unicode_bytes = if scan_unicode { bytes } else { &[] };
    let mut unicode = memchr::memchr2_iter(0xc2, 0xe2, unicode_bytes)
        .filter_map(|at| match unicode_bytes.get(at..) {
            Some([0xc2, 0x85, ..]) => Some(at + 2),
            Some([0xe2, 0x80, 0xa8 | 0xa9, ..]) => Some(at + 3),
            _ => None,
        })
        .peekable();
    // Search only possible leading bytes, without decoding every UTF-8 scalar.
    // Merge the two SIMD searches to preserve byte order without allocating.
    std::iter::from_fn(move || match (ascii.peek(), unicode.peek()) {
        (Some(a), Some(b)) if a < b => ascii.next(),
        (Some(_), None) => ascii.next(),
        _ => unicode.next(),
    })
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests {
    use super::DocumentText;
    use crate::utils::position::{line_range, offset_to_position, position_to_offset};
    use tower_lsp::lsp_types::Position;
    use vize_l0::line_index::LineBreaks;

    fn assert_matches_lsp(text: &DocumentText, source: &str) {
        assert_eq!(vize_l0::cstr!("{text}"), source);
        let starts: Vec<_> = LineBreaks::Lsp.line_starts(source).collect();
        assert_eq!(text.len_lines(), starts.len());
        assert_eq!(text.line_to_byte(text.len_lines()), source.len());
        for (line, &start) in starts.iter().enumerate() {
            assert_eq!(text.line_to_byte(line), start);
            let end = starts.get(line + 1).copied().unwrap_or(source.len());
            let rope_line = text.line(line);
            assert_eq!(vize_l0::cstr!("{rope_line}"), &source[start..end]);
            let content = source[start..end].trim_end_matches(['\r', '\n']);
            let units = content.encode_utf16().count() as u32;
            assert_eq!(
                line_range(text, line).unwrap().end,
                Position::new(line as u32, units)
            );
            for column in 0..=units + 1 {
                assert_eq!(
                    position_to_offset(text, Position::new(line as u32, column)),
                    LineBreaks::Lsp.position_to_offset(source, line as u32, column),
                    "{source:?}, line {line}, column {column}"
                );
            }
        }
        for (offset, _) in source
            .char_indices()
            .chain(std::iter::once((source.len(), '\0')))
        {
            let (line, column) = LineBreaks::Lsp.offset_to_position(source, offset);
            assert_eq!(
                offset_to_position(text, offset),
                Some(Position::new(line, column))
            );
        }
    }

    #[test]
    fn rope_and_string_coordinates_agree_for_all_line_break_conventions() {
        let pieces = [
            "", "a😀", "©€—", "\r", "\n", "\r\n", "\u{b}", "\u{c}", "\u{85}", "\u{2028}",
            "\u{2029}",
        ];
        for left in pieces {
            for right in pieces {
                let source = vize_l0::cstr!("{left}{right}é");
                assert_matches_lsp(&DocumentText::new(&source), &source);
            }
        }
    }

    #[test]
    fn replacements_keep_exceptional_boundaries_attached_to_the_text() {
        let source = "a\u{2028}😀\r\n\u{85}z\u{b}\u{c}\u{2029}";
        let original = DocumentText::new(source);
        let boundaries: Vec<_> = source
            .char_indices()
            .map(|(at, _)| at)
            .chain(std::iter::once(source.len()))
            .collect();
        for start in 0..boundaries.len() {
            for end in start..boundaries.len() {
                for inserted in ["", "X", "\r", "\n", "\u{2028}Q\u{85}"] {
                    let mut text = original.clone();
                    text.replace(start..end, inserted);
                    let expected = vize_l0::cstr!(
                        "{}{}{}",
                        &source[..boundaries[start]],
                        inserted,
                        &source[boundaries[end]..]
                    );
                    assert_matches_lsp(&text, &expected);
                }
            }
        }
        assert_matches_lsp(&original, source);
    }

    #[test]
    fn joining_and_splitting_crlf_recomputes_the_exceptional_lines() {
        let mut text = DocumentText::new("a\r\u{2028}\nb\u{2029}c");
        text.replace(2..3, "");
        assert_matches_lsp(&text, "a\r\nb\u{2029}c");
        text.replace(2..2, "\u{85}");
        assert_matches_lsp(&text, "a\r\u{85}\nb\u{2029}c");
        text.replace(0..7, "normal\ntext");
        assert_matches_lsp(&text, "normal\ntext");
        assert!(text.ignored_breaks.is_empty());
    }

    #[test]
    fn lsp_lines_can_span_several_rope_chunks() {
        let prefix = "x".repeat(4096);
        let source = vize_l0::cstr!("{prefix}\u{2028}{prefix}\u{85}😀\r\nlast");
        let text = DocumentText::new(&source);
        assert_eq!(text.len_lines(), 2);
        assert_eq!(
            position_to_offset(&text, Position::new(0, 8196)),
            Some(source.len() - 6)
        );
        assert_eq!(
            offset_to_position(&text, source.len() - 4),
            Some(Position::new(1, 0))
        );
        assert_eq!(line_range(&text, 0).unwrap().end, Position::new(0, 8196));
    }

    #[test]
    fn ordinary_documents_need_no_exceptional_boundary_allocation() {
        let mut text = DocumentText::new("a😀\r\nb\nc");
        assert!(!text.ignored_breaks.spilled());
        text.replace(0..1, "hello\r\nworld");
        assert_matches_lsp(&text, "hello\r\nworld😀\r\nb\nc");
        assert!(text.ignored_breaks.is_empty());
        assert!(!text.ignored_breaks.spilled());
    }

    #[test]
    fn snapshot_comparisons_follow_edits_without_flattening_the_rope() {
        let prefix = "x".repeat(4096);
        let source = vize_l0::cstr!("{prefix}\u{2028}😀\r\nlast");
        let mut text = DocumentText::new(&source);
        assert!(text == source.as_str());
        assert!(text != &source[..4096]);
        text.replace(4097..4098, "😃");
        assert!(text != source.as_str());
        let edited = vize_l0::cstr!("{prefix}\u{2028}😃\r\nlast");
        assert!(text == edited.as_str());
    }
}
