//! Owned, frontend-neutral Croquis semantic graph contract.
//!
//! This module is always available, including with `--no-default-features`.
//! Parser-specific analysis and Relief compatibility adapters are separate
//! optional features that produce these values but do not define them.

mod builder;
mod types;

pub use builder::CroquisSemanticSnapshotBuilder;
pub use types::*;

impl CroquisSemanticSnapshot {
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
