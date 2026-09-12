use vize_s0::Span;

use super::{OpId, RegionId};

/// Region metadata for the flat S3 graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub id: RegionId,
    pub parent: Option<RegionId>,
    pub owner: Option<OpId>,
    pub span: Span,
}

impl Region {
    /// Root region for a program.
    #[must_use]
    pub const fn root(span: Span) -> Self {
        Self {
            id: RegionId::ROOT,
            parent: None,
            owner: None,
            span,
        }
    }

    /// A nested region owned by an op in `parent`.
    #[must_use]
    pub const fn child(id: RegionId, parent: RegionId, owner: OpId, span: Span) -> Self {
        Self {
            id,
            parent: Some(parent),
            owner: Some(owner),
            span,
        }
    }
}
