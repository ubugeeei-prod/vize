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
