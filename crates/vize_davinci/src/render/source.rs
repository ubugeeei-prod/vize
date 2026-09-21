//! The authored file a diagnostic is rendered against.
//!
//! P2-1's contract: a diagnostic keys on byte offsets only, and line/column
//! exist at render time, derived from the authored text through the S0
//! [`LineIndex`]. [`SourceFile`] builds that index once per file so every
//! diagnostic in the file reuses it.
//!
//! Offsets are normalized exactly as [`vize_s0::Span::slice`] does — clamped
//! into the text, then snapped down to a `char` boundary — so the renderer
//! underlines precisely `span.slice(text)` and never panics on a malformed
//! span.

use vize_s0::line_index::LineIndex;

use super::text;

/// A file's path and authored text, indexed for rendering.
pub struct SourceFile<'a> {
    path: &'a str,
    text: &'a str,
    index: LineIndex<'a>,
    last_line: usize,
}

impl<'a> SourceFile<'a> {
    /// Index `text`, displayed under `path`.
    #[must_use]
    pub fn new(path: &'a str, text: &'a str) -> Self {
        let index = LineIndex::new(text);
        let last_line = index.line_col(text.len()).0 as usize;
        Self {
            path,
            text,
            index,
            last_line,
        }
    }

    /// The path shown in the `-->` location.
    #[must_use]
    pub const fn path(&self) -> &'a str {
        self.path
    }

    /// The authored text.
    #[must_use]
    pub const fn text(&self) -> &'a str {
        self.text
    }

    /// `offset` clamped into the text and snapped down to a `char` boundary.
    pub(crate) fn clamp(&self, offset: u32) -> usize {
        let mut offset = (offset as usize).min(self.text.len());
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    /// The normalized `[start, end)` of a span; an inverted span is empty at
    /// its clamped end, as `Span::slice` treats it.
    pub(crate) fn range(&self, span: vize_s0::Span) -> (usize, usize) {
        let end = self.clamp(span.end);
        let start = self.clamp(span.start).min(end);
        (start, end)
    }

    /// Zero-based line containing byte `offset`.
    pub(crate) fn line_of(&self, offset: usize) -> usize {
        self.index.line_col(offset).0 as usize
    }

    /// Byte offset where `line` starts.
    pub(crate) fn line_start(&self, line: usize) -> usize {
        u32::try_from(line)
            .ok()
            .and_then(|line| self.index.line_col_to_offset(line, 0))
            .unwrap_or(self.text.len())
    }

    /// Byte offset where `line`'s content ends: before its `\n`, and before a
    /// `\r` that pairs with that `\n`.
    pub(crate) fn line_end(&self, line: usize) -> usize {
        if line >= self.last_line {
            return self.text.len();
        }
        let newline = self.line_start(line + 1) - 1;
        if newline > 0 && self.text.as_bytes()[newline - 1] == b'\r' {
            newline - 1
        } else {
            newline
        }
    }

    /// The content of `line`, without its terminator.
    pub(crate) fn line_text(&self, line: usize) -> &'a str {
        &self.text[self.line_start(line)..self.line_end(line)]
    }

    /// Display column of byte `offset` within `line`: the terminal width of
    /// the line's text before it. Offsets inside the terminator sit at the
    /// end of the content.
    pub(crate) fn column(&self, line: usize, offset: usize) -> usize {
        let start = self.line_start(line);
        let offset = offset.clamp(start, self.line_end(line));
        text::width(&self.text[start..offset])
    }

    /// One-based `(line, column)` of `offset` for the `-->` location, the
    /// column in UTF-16 code units exactly as the S0 line index and the LSP
    /// report it, so a terminal location and an editor location agree.
    pub(crate) fn location(&self, offset: usize) -> (u32, u32) {
        let (line, column) = self.index.line_col(offset);
        (line + 1, column + 1)
    }
}
