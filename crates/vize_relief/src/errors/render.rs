//! Debug rendering of compiler errors with the covered source text.
//!
//! `SourceLocation` stores a byte span, not the text it covers, so the
//! derived `Debug` for [`CompilerError`] cannot print the located source.
//! Call sites that embed a debug-formatted error list in a user-facing
//! message (the SFC template gate, the binding parse-error strings) render
//! through [`CompilerErrorWithSource`] instead, which prints the exact shape
//! the derive printed when locations still stored their covered text inline —
//! `source` included, sliced from the file's source text.
//!
//! The `line`/`column` fields printed here are derived from the source text at
//! render time. `SourceLocation` itself intentionally keeps only byte spans;
//! consumers that need display coordinates should derive them at the edge from
//! the exact source text being rendered.

use core::fmt;

use crate::relief::SourceLocation;
use vize_carton::line_index::LineIndex;

use super::CompilerError;

/// A [`CompilerError`] paired with the source text its location points into.
///
/// The `Debug` output is byte-identical to the derived `Debug` of the
/// pre-span `CompilerError` (whose `SourceLocation` carried
/// `source: String` and eager `Position`s), which diagnostic messages embed
/// verbatim.
pub struct CompilerErrorWithSource<'a> {
    error: &'a CompilerError,
    source: &'a str,
}

impl<'a> CompilerErrorWithSource<'a> {
    pub fn new(error: &'a CompilerError, source: &'a str) -> Self {
        Self { error, source }
    }

    /// Wrap every error in `errors` against the same source text, in order.
    pub fn list(errors: &'a [CompilerError], source: &'a str) -> std::vec::Vec<Self> {
        errors
            .iter()
            .map(|error| Self::new(error, source))
            .collect()
    }
}

impl fmt::Debug for CompilerErrorWithSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompilerError")
            .field("code", &self.error.code)
            .field("message", &self.error.message)
            .field(
                "loc",
                &self.error.loc.as_ref().map(|loc| LocationWithSource {
                    loc,
                    source: self.source,
                }),
            )
            .finish()
    }
}

struct LocationWithSource<'a> {
    loc: &'a SourceLocation,
    source: &'a str,
}

impl fmt::Debug for LocationWithSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceLocation")
            .field(
                "start",
                &RenderedPosition::new(self.loc.span.start, self.source),
            )
            .field(
                "end",
                &RenderedPosition::new(self.loc.span.end, self.source),
            )
            .field("source", &self.loc.span.slice(self.source))
            .finish()
    }
}

struct RenderedPosition {
    offset: u32,
    line: u32,
    column: u32,
}

impl RenderedPosition {
    fn new(offset: u32, source: &str) -> Self {
        let (line, column) = LineIndex::new(source).line_col(offset as usize);
        Self {
            offset,
            line: line + 1,
            column: column + 1,
        }
    }
}

impl fmt::Debug for RenderedPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("offset", &self.offset)
            .field("line", &self.line)
            .field("column", &self.column)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::relief::SourceLocation;

    use super::super::{CompilerError, ErrorCode};
    use super::CompilerErrorWithSource;

    #[test]
    fn debug_prints_single_line_positions() {
        let loc = SourceLocation::new(5, 9);
        let error = CompilerError::with_message(
            ErrorCode::MissingEndTag,
            "Element is missing end tag.",
            Some(loc),
        );
        let rendered = format!(
            "{:?}",
            CompilerErrorWithSource::new(&error, "<div>text</div>")
        );
        assert_eq!(
            rendered,
            "CompilerError { code: MissingEndTag, \
             message: \"Element is missing end tag.\", \
             loc: Some(SourceLocation { \
             start: Position { offset: 5, line: 1, column: 6 }, \
             end: Position { offset: 9, line: 1, column: 10 }, \
             source: \"text\" }) }"
        );
    }

    #[test]
    fn debug_prints_real_multiline_positions() {
        let loc = SourceLocation::new(10, 14);
        let error =
            CompilerError::with_message(ErrorCode::InvalidExpression, "bad expression", Some(loc));
        let rendered = format!(
            "{:?}",
            CompilerErrorWithSource::new(&error, "<div>\n  {{ bad }}\n</div>")
        );
        assert_eq!(
            rendered,
            "CompilerError { code: InvalidExpression, \
             message: \"bad expression\", \
             loc: Some(SourceLocation { \
             start: Position { offset: 10, line: 2, column: 5 }, \
             end: Position { offset: 14, line: 2, column: 9 }, \
             source: \" bad\" }) }"
        );
    }

    #[test]
    fn debug_prints_a_missing_location_as_none() {
        let error = CompilerError::with_message(ErrorCode::MissingEndTag, "x", None);
        let rendered = format!("{:?}", CompilerErrorWithSource::new(&error, "<div>"));
        assert_eq!(
            rendered,
            "CompilerError { code: MissingEndTag, message: \"x\", loc: None }"
        );
    }
}
