//! Explicit cache invalidation reports.

use crate::{InputId, ProductId, ProviderId, SourceId, SourceRevisionChange};

/// Invalidation granularity currently used by Atlas.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InvalidationPolicy {
    /// Evict all cached products for the updated source and every embedded
    /// descendant, while preserving unrelated source caches.
    SourceAndEmbeddedDescendants,
    /// Evict only products transitively affected by the changed typed input.
    CompilationInputDependents,
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
