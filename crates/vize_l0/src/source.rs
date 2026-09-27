//! Source texts and line lookup.
#![expect(clippy::todo, reason = "skeleton: #6834")]

use alloc::vec::Vec;

use crate::span::Span;

/// Dense identity of one source text in a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(pub u32);

/// One borrowed source text.
#[derive(Debug, Clone, Copy)]
pub struct Source<'src> {
    /// Identity within the session.
    pub id: SourceId,
    /// The authored text.
    pub text: &'src str,
}

impl<'src> Source<'src> {
    /// The text covered by `span`, or `None` when it is out of range or not on
    /// a char boundary.
    #[must_use]
    pub fn slice(&self, span: Span) -> Option<&'src str> {
        let _ = span;
        todo!()
    }
}

/// Line starts of one source text, for mapping offsets to line/column.
#[derive(Debug, Clone, Default)]
pub struct LineIndex {
    line_starts: Vec<u32>,
}

impl LineIndex {
    /// Index the line starts of `text`.
    #[must_use]
    pub fn new(text: &str) -> Self {
        let _ = text;
        todo!()
    }

    /// Zero-based `(line, column)` of a byte offset.
    #[must_use]
    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let _ = (offset, &self.line_starts);
        todo!()
    }
}
