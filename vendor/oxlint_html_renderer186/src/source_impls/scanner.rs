use super::*;
use crate::SourceCode;
use oxc_span::Span;
use std::sync::Arc;
/// A line-break index over a contiguous source buffer, built with a single
/// forward scan and able to answer repeated span queries without re-reading
/// the source.
///
/// [`GraphicalReportHandler`] needs one span lookup per label plus one per
/// attempted snippet merge. A `SpanScanner` scans each source byte at most
/// once and records line starts for reuse. Queries that precede the index's
/// origin fall back to a standalone [`SpanReader`].
///
/// [`GraphicalReportHandler`]: crate::handlers::GraphicalReportHandler
pub struct SpanScanner<'a> {
    pub(super) context: ContextLines,
    pub(super) index: LineIndex<'a>,
}

impl<'a> SpanScanner<'a> {
    pub fn new(input: &'a [u8], context_lines_before: usize, context_lines_after: usize) -> Self {
        Self {
            context: ContextLines::new(context_lines_before, context_lines_after),
            index: LineIndex::new(input),
        }
    }

    /// Whether this scanner indexes `input`.
    pub fn is_for(&self, input: &[u8]) -> bool {
        std::ptr::eq(self.index.input, input)
    }

    /// Read a span while scanning only source bytes no earlier query scanned.
    pub fn read_span(&mut self, span: Span) -> Option<SpanContents<'a>> {
        let request = SpanRequest::new(span);
        let cut = request.prefix_end(self.index.input);
        if self.index.is_empty() {
            self.index.init(cut, self.context.before);
        } else {
            if cut
                < self
                    .index
                    .origin()
                    .expect("a non-empty index has an origin")
            {
                return self.read_unindexed(request);
            }
            self.index.cover(cut);
            // The query's leading context must not reach lines above the
            // index origin (only possible for spans out of sorted order).
            let cut_line = self.index.line_index_of(cut);
            if cut_line - self.context.before.min(cut_line) < self.index.base_line {
                return self.read_unindexed(request);
            }
        }
        IndexedReader::new(&mut self.index, request, self.context, cut).read()
    }

    pub(super) fn read_unindexed(&self, request: SpanRequest) -> Option<SpanContents<'a>> {
        SpanReader::new(self.index.input, request, self.context).read()
    }
}

impl SourceCode for str {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// Supports borrowed source text in generic contexts.
impl SourceCode for &str {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl SourceCode for String {
    fn data(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<T: ?Sized + SourceCode> SourceCode for Arc<T> {
    fn data(&self) -> &[u8] {
        self.as_ref().data()
    }

    fn name(&self) -> Option<&str> {
        self.as_ref().name()
    }
}
