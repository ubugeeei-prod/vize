//! Stable semantic snapshot facade for downstream consumers.
//!
//! `Croquis` intentionally stores rich analyzer internals. This module projects
//! those facts into deterministic, serializable view-models that lint, LSP,
//! report, and cross-file crates can share without re-walking parser data.

mod builders;
mod losses;
mod names;
mod types;

use super::Croquis;

pub use types::*;

impl CroquisSemanticSnapshot {
    /// Build a deterministic snapshot from the current croquis facts.
    pub fn from_croquis(croquis: &Croquis) -> Self {
        Self {
            summary: croquis.semantic_summary(),
            bindings: builders::binding_snapshots(croquis),
            scopes: builders::scope_snapshots(croquis),
            template_expressions: builders::template_expression_snapshots(croquis),
            component_usages: builders::component_usage_snapshots(croquis),
            provides: builders::provide_snapshots(croquis),
            injects: builders::inject_snapshots(croquis),
            reactive_sources: builders::reactive_source_snapshots(croquis),
            reactivity_losses: losses::reactivity_loss_snapshots(croquis),
        }
    }
}

impl Croquis {
    /// Return a stable semantic snapshot facade for downstream consumers.
    #[inline]
    pub fn semantic_snapshot(&self) -> CroquisSemanticSnapshot {
        CroquisSemanticSnapshot::from_croquis(self)
    }
}
