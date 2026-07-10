//! Syntax-independent construction of the owned Croquis semantic product.

use vize_carton::{CompactString, cstr};

use super::{
    CroquisSemanticSnapshot, SemanticBindingSnapshot, SemanticComponentUsageSnapshot,
    SemanticScopeBindingSnapshot, SemanticScopeSnapshot, SemanticSourceRange,
    SemanticTemplateExpressionSnapshot,
};

/// Builder used by frontend providers that already own parsed syntax facts.
///
/// The builder accepts semantic values, never Relief or OXC nodes. This keeps
/// the Croquis product contract independent while allowing each frontend to
/// derive the same cached value without a central syntax adapter.
#[derive(Debug, Default)]
pub struct CroquisSemanticSnapshotBuilder {
    snapshot: CroquisSemanticSnapshot,
}

impl CroquisSemanticSnapshotBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Continue enriching semantic facts already derived from another source
    /// block in the same container (for example, SFC script then template).
    pub fn from_snapshot(snapshot: CroquisSemanticSnapshot) -> Self {
        Self { snapshot }
    }

    pub fn add_binding(
        &mut self,
        name: &str,
        kind: &'static str,
        category: &'static str,
        range: Option<SemanticSourceRange>,
    ) {
        let id = cstr!("binding:{name}");
        self.snapshot.bindings.push(SemanticBindingSnapshot {
            id,
            name: CompactString::new(name),
            kind,
            category,
            prop_name: None,
            needs_value_in_script: false,
            range,
        });
    }

    pub fn add_scope(
        &mut self,
        id: u32,
        parent_ids: Vec<u32>,
        kind: &'static str,
        range: SemanticSourceRange,
        bindings: Vec<SemanticScopeBindingSnapshot>,
    ) {
        self.snapshot.scopes.push(SemanticScopeSnapshot {
            id,
            parent_ids,
            kind,
            range,
            binding_count: bindings.len(),
            bindings,
        });
    }

    pub fn add_template_expression(
        &mut self,
        content: &str,
        kind: &'static str,
        range: SemanticSourceRange,
        scope_id: u32,
    ) {
        let index = self.snapshot.template_expressions.len();
        self.snapshot
            .template_expressions
            .push(SemanticTemplateExpressionSnapshot {
                id: cstr!("expression:{index}"),
                content: CompactString::new(content),
                kind,
                range,
                scope_id,
                vif_guard: None,
            });
    }

    pub fn add_component_usage(
        &mut self,
        name: &str,
        range: SemanticSourceRange,
        scope_id: u32,
        has_spread_attrs: bool,
    ) {
        let index = self.snapshot.component_usages.len();
        self.snapshot
            .component_usages
            .push(SemanticComponentUsageSnapshot {
                id: cstr!("component:{index}"),
                name: CompactString::new(name),
                range,
                scope_id,
                vif_guard: None,
                has_spread_attrs,
                props: Vec::new(),
                events: Vec::new(),
                slots: Vec::new(),
            });
    }

    /// Finish the deterministic snapshot and derive aggregate counts from the
    /// values added by this frontend.
    pub fn finish(mut self) -> CroquisSemanticSnapshot {
        self.snapshot.summary.scope_count = self.snapshot.scopes.len();
        self.snapshot.summary.scope_binding_count = self
            .snapshot
            .scopes
            .iter()
            .map(|scope| scope.binding_count)
            .sum();
        self.snapshot.summary.script_binding_count = self.snapshot.bindings.len();
        self.snapshot.summary.template_expression_count = self.snapshot.template_expressions.len();
        self.snapshot.summary.component_usage_count = self.snapshot.component_usages.len();
        self.snapshot.summary.used_component_count = self.snapshot.component_usages.len();
        self.snapshot.summary.spread_attr_component_count = self
            .snapshot
            .component_usages
            .iter()
            .filter(|usage| usage.has_spread_attrs)
            .count();
        self.snapshot
    }
}
