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
//! The `line`/`column` fields printed here reproduce the retired parser
//! tracking byte-for-byte (Davinci P1-4): the parser never populated its
//! newline table, so every stored `Position` was `line: 1, column:
//! offset + 1` regardless of the real line. Corpus oracles pin diagnostic
//! messages containing that frozen shape, so this renderer reconstructs it
//! from the offset instead of deriving the real line/column. Switching this
//! output to true `vize_carton::line_index` derivation is a recorded,
//! corpus-visible behavior change for the plan to schedule — not a byte-safe
//! cleanup.

use core::fmt;

use crate::relief::SourceLocation;

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
            .field("start", &FrozenPosition(self.loc.span.start))
            .field("end", &FrozenPosition(self.loc.span.end))
            .field("source", &self.loc.span.slice(self.source))
            .finish()
    }
}

/// Renders a byte offset in the retired `Position` debug shape.
///
/// `line: 1, column: offset + 1` is not a placeholder: it is exactly what the
/// retired parser tracking stored for every node (see the module docs), and
/// the pinned diagnostic bytes depend on it.
struct FrozenPosition(u32);

impl fmt::Debug for FrozenPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("offset", &self.0)
            .field("line", &1u32)
            .field("column", &(self.0 + 1))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::relief::SourceLocation;

    use super::super::{CompilerError, ErrorCode};
    use super::CompilerErrorWithSource;

    #[test]
    fn debug_prints_the_pre_span_derive_shape() {
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
    fn debug_prints_a_missing_location_as_none() {
        let error = CompilerError::with_message(ErrorCode::MissingEndTag, "x", None);
        let rendered = format!("{:?}", CompilerErrorWithSource::new(&error, "<div>"));
        assert_eq!(
            rendered,
            "CompilerError { code: MissingEndTag, message: \"x\", loc: None }"
        );
    }
}
