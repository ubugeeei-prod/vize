//! UTF-16 positions for UTF-8 byte offsets, visited in source order.
//!
//! The parser addresses the source in UTF-8 bytes, but a JavaScript caller
//! indexes the same text in UTF-16 code units. Every position therefore
//! carries the UTF-16 `offset` (so `source.slice(start.offset, end.offset)`
//! is the node's text), a 1-based `line` and a 1-based UTF-16 `column`, the
//! convention of `@vue/compiler-core`'s `Position`.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Position {
    pub(super) offset: u32,
    pub(super) line: u32,
    pub(super) column: u32,
}

/// A forward cursor: nondecreasing queries scan each source byte once, and a
/// backward query restarts from the beginning with the same result.
pub(super) struct Cursor<'s> {
    source: &'s str,
    byte: usize,
    position: Position,
}

impl<'s> Cursor<'s> {
    pub(super) fn new(source: &'s str) -> Self {
        Self {
            source,
            byte: 0,
            position: Self::origin(),
        }
    }

    const fn origin() -> Position {
        Position {
            offset: 0,
            line: 1,
            column: 1,
        }
    }

    /// The position of byte offset `byte`, clamped to the source and moved
    /// forward to the next character boundary.
    pub(super) fn at(&mut self, byte: u32) -> Position {
        let byte = (byte as usize).min(self.source.len());
        let byte = (byte..=self.source.len())
            .find(|at| self.source.is_char_boundary(*at))
            .unwrap_or(self.source.len());
        if byte < self.byte {
            self.byte = 0;
            self.position = Self::origin();
        }
        let segment = self.source.get(self.byte..byte).unwrap_or_default();
        if segment.is_ascii() {
            self.advance_ascii(segment.as_bytes());
        } else {
            for character in segment.chars() {
                let units = character.len_utf16() as u32;
                self.position.offset += units;
                if character == '\n' {
                    self.position.line += 1;
                    self.position.column = 1;
                } else {
                    self.position.column += units;
                }
            }
        }
        self.byte = byte;
        self.position
    }

    fn advance_ascii(&mut self, segment: &[u8]) {
        self.position.offset += segment.len() as u32;
        let mut line_start = None;
        for (at, byte) in segment.iter().enumerate() {
            if *byte == b'\n' {
                self.position.line += 1;
                line_start = Some(at + 1);
            }
        }
        match line_start {
            Some(start) => self.position.column = (segment.len() - start) as u32 + 1,
            None => self.position.column += segment.len() as u32,
        }
    }
}
