use super::super::CroquisSemanticSummary;
use serde::Serialize;
use vize_carton::CompactString;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CroquisSemanticSnapshot {
    pub summary: CroquisSemanticSummary,
    pub bindings: Vec<SemanticBindingSnapshot>,
    pub scopes: Vec<SemanticScopeSnapshot>,
    pub template_expressions: Vec<SemanticTemplateExpressionSnapshot>,
    pub component_usages: Vec<SemanticComponentUsageSnapshot>,
    pub provides: Vec<SemanticProvideSnapshot>,
    pub injects: Vec<SemanticInjectSnapshot>,
    pub reactive_sources: Vec<SemanticReactiveSourceSnapshot>,
    pub reactivity_losses: Vec<SemanticReactivityLossSnapshot>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticSourceRange {
    pub start: u32,
    pub end: u32,
}

impl SemanticSourceRange {
    #[inline]
    pub(crate) const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticBindingSnapshot {
    pub id: CompactString,
    pub name: CompactString,
    pub kind: &'static str,
    pub category: &'static str,
    pub prop_name: Option<CompactString>,
    pub needs_value_in_script: bool,
    pub range: Option<SemanticSourceRange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticScopeSnapshot {
    pub id: u32,
    pub parent_ids: Vec<u32>,
    pub kind: &'static str,
    pub range: SemanticSourceRange,
    pub binding_count: usize,
    pub bindings: Vec<SemanticScopeBindingSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticScopeBindingSnapshot {
    pub name: CompactString,
    pub kind: &'static str,
    pub declaration_offset: u32,
    pub used: bool,
    pub mutated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticTemplateExpressionSnapshot {
    pub id: CompactString,
    pub content: CompactString,
    pub kind: &'static str,
    pub range: SemanticSourceRange,
    pub scope_id: u32,
    pub vif_guard: Option<CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticComponentUsageSnapshot {
    pub id: CompactString,
    pub name: CompactString,
    pub range: SemanticSourceRange,
    pub scope_id: u32,
    pub vif_guard: Option<CompactString>,
    pub has_spread_attrs: bool,
    pub props: Vec<SemanticPassedPropSnapshot>,
    pub events: Vec<SemanticEventListenerSnapshot>,
    pub slots: Vec<SemanticSlotUsageSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticPassedPropSnapshot {
    pub name: CompactString,
    pub value: Option<CompactString>,
    pub range: SemanticSourceRange,
    pub dynamic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticEventListenerSnapshot {
    pub name: CompactString,
    pub handler: Option<CompactString>,
    pub modifiers: Vec<CompactString>,
    pub range: SemanticSourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticSlotUsageSnapshot {
    pub name: CompactString,
    pub scope_vars: Vec<CompactString>,
    pub range: SemanticSourceRange,
    pub scoped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticProvideSnapshot {
    pub id: CompactString,
    pub key: CompactString,
    pub key_kind: &'static str,
    pub value: CompactString,
    pub value_type: Option<CompactString>,
    pub from_composable: Option<CompactString>,
    pub range: SemanticSourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticInjectSnapshot {
    pub id: CompactString,
    pub key: CompactString,
    pub key_kind: &'static str,
    pub local_name: CompactString,
    pub default_value: Option<CompactString>,
    pub expected_type: Option<CompactString>,
    pub pattern: &'static str,
    pub destructured_names: Vec<CompactString>,
    pub from_composable: Option<CompactString>,
    pub range: SemanticSourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticReactiveSourceSnapshot {
    pub id: CompactString,
    pub name: CompactString,
    pub kind: &'static str,
    pub category: &'static str,
    pub needs_value_access: bool,
    pub declaration_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticReactivityLossSnapshot {
    pub id: CompactString,
    pub kind: &'static str,
    pub category: &'static str,
    pub source_name: Option<CompactString>,
    pub target_name: Option<CompactString>,
    pub property_name: Option<CompactString>,
    pub extracted_names: Vec<CompactString>,
    pub range: SemanticSourceRange,
}
