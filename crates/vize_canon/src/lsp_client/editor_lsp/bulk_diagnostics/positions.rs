//! UTF-16 coordinates from the acknowledged, owner-retained document text.

use lsp_types::{Position, Range};

pub(super) struct Positions {
    lines: Vec<u32>,
    surrogate_interiors: Vec<u32>,
    length: u32,
}

impl Positions {
    pub(super) fn new(text: &str) -> Option<Self> {
        let mut lines = vec![0];
        let mut surrogate_interiors = Vec::new();
        let mut length = 0u32;
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch.len_utf16() == 2 {
                surrogate_interiors.push(length.checked_add(1)?);
            }
            length = length.checked_add(u32::try_from(ch.len_utf16()).ok()?)?;
            if ch == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
                length = length.checked_add(1)?;
                lines.push(length);
            } else if matches!(ch, '\r' | '\n') {
                lines.push(length);
            }
        }
        Some(Self {
            lines,
            surrogate_interiors,
            length,
        })
    }

    fn position(&self, offset: u32) -> Option<Position> {
        if offset > self.length || self.surrogate_interiors.binary_search(&offset).is_ok() {
            return None;
        }
        let line = self
            .lines
            .partition_point(|start| *start <= offset)
            .checked_sub(1)?;
        Some(Position::new(
            u32::try_from(line).ok()?,
            offset - self.lines.get(line)?,
        ))
    }

    pub(super) fn range(&self, start: u32, end: u32) -> Option<Range> {
        if start > end {
            return None;
        }
        Some(Range::new(self.position(start)?, self.position(end)?))
    }
}
