//! Protocol-neutral source coordinates for UTF-16 consumers.
//!
//! These value types do not own URIs, diagnostics, requests or transport state.
//! Their field names and serialization retain the existing LSP-compatible
//! contract; host adapters can re-export them without conversion or identity
//! changes.

use serde::{Deserialize, Serialize};

/// Zero-based source position with a UTF-16 code-unit column.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Source range expressed in zero-based UTF-16 positions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn from_positions(start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> Self {
        Self {
            start: Position::new(start_line, start_char),
            end: Position::new(end_line, end_char),
        }
    }
}

#[cfg(feature = "positioned-edits")]
impl Position {
    /// Position after a UTF-8 slice beginning here, within a source whose
    /// offsets fit in `u32`. A carriage return occupies one UTF-16 column.
    #[inline]
    pub fn after_text(self, text: &str) -> Self {
        use super::utf16::utf16_len;

        let bytes = text.as_bytes();
        match bytes {
            [] => self,
            [b'\n'] => Self::new(self.line + 1, 0),
            [_] => Self::new(self.line, self.character + 1),
            _ => {
                let mut breaks = memchr::memchr_iter(b'\n', bytes);
                let Some(last) = breaks.next_back() else {
                    return Self::new(self.line, self.character + utf16_len(text) as u32);
                };
                let (_, tail) = text.split_at(last + 1);
                Self::new(
                    self.line + 1 + breaks.count() as u32,
                    utf16_len(tail) as u32,
                )
            }
        }
    }
}

#[cfg(feature = "positioned-edits")]
impl Range {
    /// Locate the suffix beginning at `split` in a prefix whose first byte is
    /// at `origin`. Both endpoints share a scan of the last line. A split
    /// inside a scalar counts it, and a split beyond the prefix selects EOF.
    #[inline]
    pub fn suffix_of(origin: Position, prefix: &str, split: usize) -> Self {
        use super::utf16::utf16_len;

        if split == 0 {
            return Self::new(origin, origin.after_text(prefix));
        }
        let split = prefix.ceil_char_boundary(split);
        if split == prefix.len() {
            let at = origin.after_text(prefix);
            return Self::new(at, at);
        }
        let mut breaks = memchr::memchr_iter(b'\n', prefix.as_bytes());
        let (line_origin, tail, in_line) = match breaks.next_back() {
            None => (origin, prefix, split),
            Some(last) => {
                let line = origin.line + 1 + breaks.count() as u32;
                let (_, tail) = prefix.split_at(last + 1);
                if last >= split {
                    let (before, _) = prefix.split_at(split);
                    return Self::new(
                        origin.after_text(before),
                        Position::new(line, utf16_len(tail) as u32),
                    );
                }
                (Position::new(line, 0), tail, split - (last + 1))
            }
        };
        let (before, total) = if tail.is_ascii() {
            (in_line, tail.len())
        } else {
            let (left, right) = tail.split_at(in_line);
            let before = utf16_len(left);
            (before, before + utf16_len(right))
        };
        Self::new(
            Position::new(line_origin.line, line_origin.character + before as u32),
            Position::new(line_origin.line, line_origin.character + total as u32),
        )
    }
}

#[cfg(all(test, feature = "positioned-edits"))]
mod tests {
    use super::{Position, Range};
    use crate::line_index::offset_to_position;

    #[test]
    fn suffix_ranges_match_original_positions_at_every_split_and_start_line() {
        for source in [
            "\r\nα雪🍊x\n\r\n\u{2028}",
            "first\nsecond\nthird",
            "",
            "abc",
        ] {
            for begin in (0..=source.len()).filter(|at| source.is_char_boundary(*at)) {
                let origin = offset_to_position(source, begin);
                for end in (begin..=source.len()).filter(|at| source.is_char_boundary(*at)) {
                    let prefix = &source[begin..end];
                    for split in 0..=prefix.len() + 1 {
                        let expected = Range::new(
                            offset_to_position(source, begin + split.min(prefix.len())),
                            offset_to_position(source, end),
                        );
                        assert_eq!(Range::suffix_of(origin, prefix, split), expected);
                    }
                }
            }
        }
        assert_eq!(
            Range::suffix_of(Position::default(), "雪", usize::MAX),
            Range::from_positions(0, 1, 0, 1)
        );
    }
}

#[cfg(feature = "positioned-edits")]
impl Position {
    /// Advance over a complete LSP segment. CRLF is one break and its two
    /// bytes must stay in the same segment.
    #[inline]
    pub fn after_lsp_segment(self, text: &str) -> Self {
        use super::utf16::utf16_len;
        let Some(last) = memchr::memrchr2(b'\r', b'\n', text.as_bytes()) else {
            return Self::new(self.line, self.character + utf16_len(text) as u32);
        };
        let (lines, tail) = text.split_at(last + 1);
        Self::new(
            self.line + str_indices::lines_crlf::count_breaks(lines) as u32,
            utf16_len(tail) as u32,
        )
    }
}

#[cfg(feature = "positioned-edits")]
impl Range {
    /// Locate both ends of an edit in an original source prefix. The caller
    /// keeps CRLF pairs together at both endpoints. Within a scalar, use the
    /// next valid UTF-8 boundary, as in the original offset conversions.
    #[inline]
    pub fn lsp_suffix(prefix: &str, split: usize) -> Self {
        use super::utf16::utf16_len;

        let split = prefix.ceil_char_boundary(split);
        let (line, line_start) = match memchr::memrchr2(b'\r', b'\n', prefix.as_bytes()) {
            Some(last) => {
                let (lines, _) = prefix.split_at(last + 1);
                (
                    str_indices::lines_crlf::count_breaks(lines) as u32,
                    last + 1,
                )
            }
            None => (0, 0),
        };
        let (_, tail) = prefix.split_at(line_start);
        if split < line_start {
            let (before, _) = prefix.split_at(split);
            return Self::new(
                Position::default().after_lsp_segment(before),
                Position::new(line, utf16_len(tail) as u32),
            );
        }
        let column = split - line_start;
        let (start, end) = if tail.is_ascii() {
            (column, tail.len())
        } else {
            let (first, rest) = tail.split_at(column);
            let start = utf16_len(first);
            (start, start + utf16_len(rest))
        };
        Self::from_positions(line, start as u32, line, end as u32)
    }
}
