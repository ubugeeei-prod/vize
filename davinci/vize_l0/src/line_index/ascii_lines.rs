//! LSP positions in an ASCII prefix with one shared scan of its line breaks.
use super::Position;

/// A cursor over a checked ASCII prefix whose byte offsets fit the IR span space.
pub struct AsciiLineCursor<'s> {
    source: &'s [u8],
    breaks: memchr::Memchr<'s>,
    next_break: usize,
    line: u32,
    line_start: usize,
}

impl<'s> AsciiLineCursor<'s> {
    /// ASCII byte and UTF-16 columns agree throughout the prefix.
    #[inline]
    pub fn new(source: &'s str) -> Option<Self> {
        let bytes = source.as_bytes();
        if !bytes.is_ascii() || bytes.len() > u32::MAX as usize {
            return None;
        }
        Some(Self::from_ascii(bytes))
    }

    #[inline]
    fn from_ascii(source: &'s [u8]) -> Self {
        let mut breaks = memchr::memchr_iter(b'\n', source);
        let next_break = breaks.next().unwrap_or(usize::MAX);
        Self {
            source,
            breaks,
            next_break,
            line: 0,
            line_start: 0,
        }
    }

    /// Reuse previously visited line breaks. Backward requests restart only
    /// when they cross the beginning of the current line. Offsets past EOF
    /// name the end of the prefix, as in the single-shot converter.
    #[inline]
    pub fn position(&mut self, byte: u32) -> Position {
        let byte = (byte as usize).min(self.source.len());
        if byte < self.line_start {
            *self = Self::from_ascii(self.source);
        }
        while self.next_break < byte {
            self.line += 1;
            self.line_start = self.next_break + 1;
            self.next_break = self.breaks.next().unwrap_or(usize::MAX);
        }
        Position::new(self.line, (byte - self.line_start) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::AsciiLineCursor;
    use crate::line_index::offset_to_position;

    #[test]
    fn repeated_forward_and_backward_requests_match_original_positions() {
        for source in ["", "a", "\n", "abc\r\n\n\rxy\n"] {
            let Some(mut cursor) = AsciiLineCursor::new(source) else {
                panic!("ASCII fixture");
            };
            let end = source.len() as u32;
            for at in (0..=end).chain((0..=end).rev()).chain([u32::MAX, 0]) {
                assert_eq!(cursor.position(at), offset_to_position(source, at as usize));
            }
        }
        assert!(AsciiLineCursor::new("x\n雪").is_none());
    }
}
