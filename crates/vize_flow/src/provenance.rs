use vize_carton::source_range::SourceRange;

use crate::SourceId;

/// A byte span in one source registered with a [`crate::FlowGraph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    source: SourceId,
    range: SourceRange,
}

impl SourceSpan {
    /// Create a source span.
    #[inline]
    pub const fn new(source: SourceId, range: SourceRange) -> Self {
        Self { source, range }
    }

    /// Source containing the range.
    #[inline]
    pub const fn source(self) -> SourceId {
        self.source
    }

    /// Half-open byte range in the source.
    #[inline]
    pub const fn range(self) -> SourceRange {
        self.range
    }
}

/// Origin of a flow entity.
///
/// Synthetic entities make transformations explicit instead of assigning
/// misleading source ranges to compiler-created blocks or values.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provenance {
    /// Created by analysis or a transform and not directly written in source.
    #[default]
    Synthetic,
    /// Directly attributable to a source byte range.
    Source(SourceSpan),
}

impl Provenance {
    /// Construct source-backed provenance.
    #[inline]
    pub const fn source(source: SourceId, range: SourceRange) -> Self {
        Self::Source(SourceSpan::new(source, range))
    }

    /// Return the source span, if this entity is source-backed.
    #[inline]
    pub const fn span(self) -> Option<SourceSpan> {
        match self {
            Self::Synthetic => None,
            Self::Source(span) => Some(span),
        }
    }
}
