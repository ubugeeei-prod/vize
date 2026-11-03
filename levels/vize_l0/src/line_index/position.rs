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
