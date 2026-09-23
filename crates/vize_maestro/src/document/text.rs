//! LSP line boundaries over Ropey's Unicode line metrics.

use std::{fmt, ops::Range};

use ropey::{Rope, RopeSlice};
use vize_s0::SmallVec;

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
mod tests;
