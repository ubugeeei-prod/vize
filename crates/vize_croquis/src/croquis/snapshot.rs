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

    /// Find a binding by its declared or macro-exposed name.
    #[inline]
    pub fn binding_by_name(&self, name: &str) -> Option<&SemanticBindingSnapshot> {
        self.bindings.iter().find(|binding| binding.name == name)
    }

    /// Find a scope by its stable numeric ID.
    #[inline]
    pub fn scope_by_id(&self, id: u32) -> Option<&SemanticScopeSnapshot> {
        self.scopes.iter().find(|scope| scope.id == id)
    }

    /// Iterate component usages with the exact template component name.
    #[inline]
    pub fn component_usages_by_name<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a SemanticComponentUsageSnapshot> + 'a {
        self.component_usages
            .iter()
            .filter(move |usage| usage.name == name)
    }

    /// Iterate template expressions that were analyzed in one scope.
    #[inline]
    pub fn template_expressions_in_scope(
        &self,
        scope_id: u32,
    ) -> impl Iterator<Item = &SemanticTemplateExpressionSnapshot> {
        self.template_expressions
            .iter()
            .filter(move |expression| expression.scope_id == scope_id)
    }

    /// Iterate provides with the given normalized key text.
    #[inline]
    pub fn provides_by_key<'a>(
        &'a self,
        key: &'a str,
    ) -> impl Iterator<Item = &'a SemanticProvideSnapshot> + 'a {
        self.provides
            .iter()
            .filter(move |provide| provide.key == key)
    }

    /// Iterate injects with the given normalized key text.
    #[inline]
    pub fn injects_by_key<'a>(
        &'a self,
        key: &'a str,
    ) -> impl Iterator<Item = &'a SemanticInjectSnapshot> + 'a {
        self.injects.iter().filter(move |inject| inject.key == key)
    }

    /// Find a reactive source by binding name.
    #[inline]
    pub fn reactive_source_by_name(&self, name: &str) -> Option<&SemanticReactiveSourceSnapshot> {
        self.reactive_sources
            .iter()
            .find(|source| source.name == name)
    }
}

impl Croquis {
    /// Return a stable semantic snapshot facade for downstream consumers.
    #[inline]
    pub fn semantic_snapshot(&self) -> CroquisSemanticSnapshot {
        CroquisSemanticSnapshot::from_croquis(self)
    }
}
