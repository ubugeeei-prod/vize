//! Mapping OXC byte spans to Vize [`SourceLocation`]s.
//!
//! The lowering layer must preserve enough source information for compiler
//! diagnostics, the type checker, the LSP, and Patina fixes. OXC reports byte
//! offsets only; Vize's [`SourceLocation`] additionally carries 1-indexed
//! line/column positions (mirroring the template parser's convention) plus the
//! original source slice. This module is the single home for that conversion.

use oxc_span::Span;
use vize_carton::line_index::LineIndex;
use vize_relief::ast::core::{Position, SourceLocation};

/// Converts OXC byte spans into Vize source locations against one source text.
///
/// Build it once per module and reuse it: the underlying [`LineIndex`] turns
/// each position lookup into a binary search over line starts rather than a
/// full scan of the source.
pub struct SpanMapper<'s> {
    source: &'s str,
    line_index: LineIndex<'s>,
}

impl<'s> SpanMapper<'s> {
    /// Build a span mapper for `source`.
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            line_index: LineIndex::new(source),
        }
    }

    /// The source text this mapper indexes.
    pub fn source(&self) -> &'s str {
        self.source
    }

    /// Convert a byte offset to a 1-indexed [`Position`].
    ///
    /// [`LineIndex`] reports 0-indexed LSP coordinates (UTF-16 columns); we add
    /// one to each axis so the resulting [`Position`] matches the 1-indexed
    /// convention the rest of the relief AST uses.
    pub fn position(&self, offset: u32) -> Position {
        let (line, column) = self.line_index.line_col(offset as usize);
        Position::new(offset, line + 1, column + 1)
    }

    /// The source slice covered by `span`, clamped to the source bounds.
    pub fn slice(&self, span: Span) -> &'s str {
        let start = (span.start as usize).min(self.source.len());
        let end = (span.end as usize).min(self.source.len()).max(start);
        &self.source[start..end]
    }

    /// Convert an OXC [`Span`] to a full [`SourceLocation`].
    pub fn location(&self, span: Span) -> SourceLocation {
        SourceLocation::new(
            self.position(span.start),
            self.position(span.end),
            self.slice(span),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_is_one_indexed() {
        let mapper = SpanMapper::new("ab\ncd");
        // offset 0 -> line 1, col 1
        let p = mapper.position(0);
        assert_eq!((p.line, p.column), (1, 1));
        // offset 3 is 'c' on the second line -> line 2, col 1
        let p = mapper.position(3);
        assert_eq!((p.line, p.column), (2, 1));
    }

    #[test]
    fn slice_extracts_substring() {
        let mapper = SpanMapper::new("<div>hi</div>");
        let span = Span::new(5, 7);
        assert_eq!(mapper.slice(span), "hi");
    }

    #[test]
    fn slice_clamps_out_of_range() {
        let mapper = SpanMapper::new("abc");
        assert_eq!(mapper.slice(Span::new(2, 99)), "c");
        assert_eq!(mapper.slice(Span::new(99, 99)), "");
    }

    #[test]
    fn location_records_offsets_and_source() {
        let mapper = SpanMapper::new("x = <a/>");
        let loc = mapper.location(Span::new(4, 8));
        assert_eq!(loc.start.offset, 4);
        assert_eq!(loc.end.offset, 8);
        assert_eq!(loc.source.as_str(), "<a/>");
    }
}
