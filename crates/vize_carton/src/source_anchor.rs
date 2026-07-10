//! Domain-neutral source identity carried by compiler artifacts.

use serde::{Deserialize, Serialize};

use crate::source_range::SourceRange;

/// Stable identity and revision of the source that owns an artifact range.
///
/// The numeric identity is assigned by the compilation host. Representation
/// crates deliberately treat it as opaque, so they can preserve provenance
/// without depending on the execution engine that allocated the source.
#[derive(Debug, Clone, Copy, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SourceAnchor {
    source: u64,
    revision: u64,
    parent_range: Option<SourceRange>,
}

impl SourceAnchor {
    /// Anchor an artifact directly in one source revision.
    pub const fn new(source: u64, revision: u64) -> Self {
        Self {
            source,
            revision,
            parent_range: None,
        }
    }

    /// Record the containing range when an artifact represents an embedded
    /// region such as an SFC template block.
    pub const fn with_parent_range(mut self, range: SourceRange) -> Self {
        self.parent_range = Some(range);
        self
    }

    /// Opaque source identity assigned by the compilation host.
    pub const fn source(self) -> u64 {
        self.source
    }

    /// Exact source revision from which the artifact was derived.
    pub const fn revision(self) -> u64 {
        self.revision
    }

    /// Containing range in the owning source for an embedded artifact.
    pub const fn parent_range(self) -> Option<SourceRange> {
        self.parent_range
    }

    /// Resolve an artifact-local range into the owning source coordinate space.
    pub const fn resolve_range(self, local: SourceRange) -> SourceRange {
        let offset = match self.parent_range {
            Some(range) => range.start,
            None => 0,
        };
        SourceRange::new(
            offset.saturating_add(local.start),
            offset.saturating_add(local.end),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_anchor_resolves_local_ranges_without_losing_identity() {
        let anchor = SourceAnchor::new(7, 3).with_parent_range(SourceRange::new(100, 180));

        assert_eq!(anchor.source(), 7);
        assert_eq!(anchor.revision(), 3);
        assert_eq!(anchor.parent_range(), Some(SourceRange::new(100, 180)));
        assert_eq!(
            anchor.resolve_range(SourceRange::new(4, 9)),
            SourceRange::new(104, 109)
        );
    }
}
