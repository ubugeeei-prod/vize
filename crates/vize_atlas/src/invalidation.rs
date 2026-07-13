//! Explicit cache invalidation reports.

use crate::{InputId, ProductId, ProviderId, SourceId, SourceInputId, SourceRevisionChange};

/// Invalidation granularity currently used by Atlas.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InvalidationPolicy {
    /// Evict all cached products for the updated source and every embedded
    /// descendant, while preserving unrelated source caches.
    SourceAndEmbeddedDescendants,
    /// Remove a source subtree and every cached product that owns or depends
    /// on any source in that subtree.
    RemovedSourceAndEmbeddedDescendants,
    /// Evict only products transitively affected by the changed typed input.
    CompilationInputDependents,
    /// Evict only products whose closure read one source-scoped input.
    SourceInputDependents,
}

/// Result of installing or replacing one typed source-scoped input.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SourceInputInvalidationReport {
    source: SourceId,
    input: SourceInputId,
    replaced: bool,
    evicted: Vec<InvalidatedProduct>,
}

impl SourceInputInvalidationReport {
    pub(crate) const fn new(
        source: SourceId,
        input: SourceInputId,
        replaced: bool,
        evicted: Vec<InvalidatedProduct>,
    ) -> Self {
        Self {
            source,
            input,
            replaced,
            evicted,
        }
    }

    pub const fn source(&self) -> SourceId {
        self.source
    }

    pub const fn input(&self) -> SourceInputId {
        self.input
    }

    pub const fn replaced(&self) -> bool {
        self.replaced
    }

    pub const fn policy(&self) -> InvalidationPolicy {
        InvalidationPolicy::SourceInputDependents
    }

    pub fn evicted(&self) -> &[InvalidatedProduct] {
        &self.evicted
    }
}

/// Observable result of closing or deleting a source from a compilation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SourceRemovalReport {
    removed: Vec<SourceId>,
    evicted: Vec<InvalidatedProduct>,
}

impl SourceRemovalReport {
    pub(crate) const fn new(removed: Vec<SourceId>, evicted: Vec<InvalidatedProduct>) -> Self {
        Self { removed, evicted }
    }

    /// Removed root followed by its embedded descendants in stable tree order.
    pub fn removed(&self) -> &[SourceId] {
        &self.removed
    }

    pub const fn policy(&self) -> InvalidationPolicy {
        InvalidationPolicy::RemovedSourceAndEmbeddedDescendants
    }

    /// Cached products owned by or dependent on a removed source.
    pub fn evicted(&self) -> &[InvalidatedProduct] {
        &self.evicted
    }
}

/// Observable result of installing or replacing one typed compilation input.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InputInvalidationReport {
    input: InputId,
    replaced: bool,
    evicted: Vec<InvalidatedProduct>,
}

impl InputInvalidationReport {
    pub(crate) const fn new(
        input: InputId,
        replaced: bool,
        evicted: Vec<InvalidatedProduct>,
    ) -> Self {
        Self {
            input,
            replaced,
            evicted,
        }
    }

    pub const fn input(&self) -> InputId {
        self.input
    }

    pub const fn replaced(&self) -> bool {
        self.replaced
    }

    pub const fn policy(&self) -> InvalidationPolicy {
        InvalidationPolicy::CompilationInputDependents
    }

    pub fn evicted(&self) -> &[InvalidatedProduct] {
        &self.evicted
    }
}

/// One cache entry evicted by a source update.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InvalidatedProduct {
    /// Source whose cached product was evicted.
    pub source: SourceId,
    /// Product whose cached value was evicted.
    pub product: ProductId,
    /// Provider that had produced the cached value.
    pub provider: ProviderId,
}

/// Observable result of updating a source.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InvalidationReport {
    updated: SourceId,
    policy: InvalidationPolicy,
    revisions: Vec<SourceRevisionChange>,
    evicted: Vec<InvalidatedProduct>,
}

impl InvalidationReport {
    pub(crate) const fn new(
        updated: SourceId,
        revisions: Vec<SourceRevisionChange>,
        evicted: Vec<InvalidatedProduct>,
    ) -> Self {
        Self {
            updated,
            policy: InvalidationPolicy::SourceAndEmbeddedDescendants,
            revisions,
            evicted,
        }
    }

    pub const fn updated(&self) -> SourceId {
        self.updated
    }

    pub const fn policy(&self) -> InvalidationPolicy {
        self.policy
    }

    /// Revisions changed for the source and its provenance descendants.
    pub fn revisions(&self) -> &[SourceRevisionChange] {
        &self.revisions
    }

    /// Concrete cached products removed by the update.
    pub fn evicted(&self) -> &[InvalidatedProduct] {
        &self.evicted
    }
}
