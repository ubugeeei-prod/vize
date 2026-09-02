//! Mapping OXC byte spans to Vize [`SourceLocation`]s.
//!
//! The lowering layer must preserve enough source information for compiler
//! diagnostics, the type checker, the LSP, and Patina fixes. OXC reports byte
//! offsets and Vize's [`SourceLocation`] is a byte span too (Davinci P1-4
//! retired the eager line/column fields), so the conversion is a direct
//! offset carry-over; consumers that render line/column derive them from the
//! offsets at their edge via `vize_s0::line_index`. This module is the
//! single home for that conversion.

use oxc_span::Span;
use vize_relief::SourceLocation;

/// Converts OXC byte spans into Vize source locations against one source text.
pub struct SpanMapper<'s> {
    source: &'s str,
}

impl<'s> SpanMapper<'s> {
    /// Build a span mapper for `source`.
    pub fn new(source: &'s str) -> Self {
        Self { source }
    }

    /// The source text this mapper indexes.
    pub fn source(&self) -> &'s str {
        self.source
    }

    /// The source slice covered by `span`, clamped to the source bounds.
    pub fn slice(&self, span: Span) -> &'s str {
        let start = (span.start as usize).min(self.source.len());
        let end = (span.end as usize).min(self.source.len()).max(start);
        &self.source[start..end]
    }

    /// Convert an OXC [`Span`] to a full [`SourceLocation`].
    pub fn location(&self, span: Span) -> SourceLocation {
        SourceLocation::new(span.start, span.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn location_records_offsets() {
        let mapper = SpanMapper::new("x = <a/>");
        let loc = mapper.location(Span::new(4, 8));
        assert_eq!(loc.span.start, 4);
        assert_eq!(loc.span.end, 8);
        assert_eq!(loc.span.slice(mapper.source()), "<a/>");
    }
}
