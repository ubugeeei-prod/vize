//! Line/column positions in **Unicode scalar values**, `moonc`'s unit.
//!
//! S0 spans are UTF-8 byte offsets and the LSP speaks UTF-16
//! (`vize_s0::line_index`); `moonc` reports 1-based lines and 1-based,
//! end-exclusive columns counted in Unicode scalar values (measured in
//! P6-4a: `"😀x"` advances the column by two, not three or five). The
//! dialect converts at its boundary, in this one place.

/// A 1-based line and a 1-based column in Unicode scalar values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineCol {
    /// 1-based line.
    pub line: u32,
    /// 1-based column, in Unicode scalar values.
    pub col: u32,
}

/// Line starts of one text, for conversions in both directions.
#[derive(Debug, Clone)]
pub struct Lines<'a> {
    text: &'a str,
    starts: Vec<usize>,
}

impl<'a> Lines<'a> {
    /// Index `text`'s line starts (`\n` terminates a line).
    #[must_use]
    pub fn new(text: &'a str) -> Self {
        let mut starts = vec![0];
        starts.extend(
            text.bytes()
                .enumerate()
                .filter(|&(_, byte)| byte == b'\n')
                .map(|(at, _)| at + 1),
        );
        Self { text, starts }
    }

    /// The byte offset of `at`, or `None` when the position lies outside
    /// the text. A column one past the line's last scalar (the
    /// end-exclusive position) is valid.
    #[must_use]
    pub fn offset(&self, at: LineCol) -> Option<usize> {
        let line = usize::try_from(at.line).ok()?.checked_sub(1)?;
        let start = *self.starts.get(line)?;
        let end = self
            .starts
            .get(line + 1)
            .map_or(self.text.len(), |next| next - 1);
        let steps = usize::try_from(at.col).ok()?.checked_sub(1)?;
        let line_text = self.text.get(start..end)?;
        if steps == line_text.chars().count() {
            return Some(end);
        }
        line_text
            .char_indices()
            .nth(steps)
            .map(|(offset, _)| start + offset)
    }

    /// The position of byte `offset` (clamped to the text; an offset
    /// inside a scalar resolves to that scalar).
    #[must_use]
    pub fn position(&self, offset: usize) -> LineCol {
        let offset = offset.min(self.text.len());
        let line = self
            .starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);
        let start = self.starts.get(line).copied().unwrap_or(0);
        let col = self
            .text
            .get(start..)
            .unwrap_or_default()
            .char_indices()
            .take_while(|&(at, _)| start + at < offset)
            .count();
        LineCol {
            line: u32::try_from(line + 1).unwrap_or(u32::MAX),
            col: u32::try_from(col + 1).unwrap_or(u32::MAX),
        }
    }

    /// The text of 1-based `line`, without its terminator.
    #[must_use]
    pub fn line_text(&self, line: u32) -> &'a str {
        let Some(index) = usize::try_from(line)
            .ok()
            .and_then(|line| line.checked_sub(1))
        else {
            return "";
        };
        let Some(&start) = self.starts.get(index) else {
            return "";
        };
        let end = self
            .starts
            .get(index + 1)
            .map_or(self.text.len(), |next| next - 1);
        self.text
            .get(start..end)
            .unwrap_or_default()
            .trim_end_matches('\r')
    }
}

#[cfg(test)]
mod tests {
    use super::{LineCol, Lines};

    #[test]
    fn columns_count_unicode_scalars_both_ways() {
        let text = "ab\n  let s = \"😀x\"; if s\n";
        let lines = Lines::new(text);
        // The P6-4a measurement: moonc put `s` at 2:20.
        let s = text.rfind('s').unwrap();
        assert_eq!(lines.position(s), LineCol { line: 2, col: 20 });
        assert_eq!(lines.offset(LineCol { line: 2, col: 20 }), Some(s));
        assert_eq!(lines.offset(LineCol { line: 2, col: 21 }), Some(s + 1));
        assert_eq!(lines.offset(LineCol { line: 2, col: 22 }), None);
        assert_eq!(lines.offset(LineCol { line: 3, col: 1 }), Some(text.len()));
        assert_eq!(lines.offset(LineCol { line: 4, col: 1 }), None);
        assert_eq!(lines.offset(LineCol { line: 0, col: 1 }), None);
        assert_eq!(lines.line_text(2), "  let s = \"😀x\"; if s");
    }

    #[test]
    fn every_scalar_boundary_round_trips() {
        let text = "é😀\nx\r\n中文";
        let lines = Lines::new(text);
        for (offset, _) in text.char_indices().chain([(text.len(), ' ')]) {
            assert_eq!(
                lines.offset(lines.position(offset)),
                Some(offset),
                "{offset}"
            );
        }
    }
}
