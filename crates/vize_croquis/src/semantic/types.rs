use serde::Serialize;
use vize_carton::{CompactString, source_anchor::SourceAnchor};

/// Aggregate counts for one owned semantic snapshot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CroquisSemanticSummary {
    pub scope_count: usize,
    pub scope_binding_count: usize,
    pub template_scope_count: usize,
    pub v_for_scope_count: usize,
    pub v_slot_scope_count: usize,
    pub event_handler_scope_count: usize,
    pub callback_scope_count: usize,
    pub symbol_count: usize,
    pub symbol_reference_count: usize,
    pub unused_symbol_count: usize,
    pub script_binding_count: usize,
    pub prop_alias_count: usize,
    pub macro_call_count: usize,
    pub prop_definition_count: usize,
    pub emit_definition_count: usize,
    pub emit_call_count: usize,
    pub model_definition_count: usize,
    pub exposed_binding_count: usize,
    pub slot_definition_count: usize,
    pub top_level_await_count: usize,
    pub hoist_count: usize,
    pub reactive_source_count: usize,
    pub reactivity_loss_count: usize,
    pub race_condition_count: usize,
    pub provide_count: usize,
    pub inject_count: usize,
    pub destructured_inject_count: usize,
    pub composable_count: usize,
    pub setup_context_violation_count: usize,
    pub used_component_count: usize,
    pub component_registration_count: usize,
    pub component_usage_count: usize,
    pub passed_prop_count: usize,
    pub event_listener_count: usize,
    pub slot_usage_count: usize,
    pub spread_attr_component_count: usize,
    pub used_directive_count: usize,
    pub template_expression_count: usize,
    pub v_if_expression_count: usize,
    pub v_model_expression_count: usize,
    pub element_id_count: usize,
    pub static_element_id_count: usize,
    pub dynamic_element_id_count: usize,
    pub id_definition_count: usize,
    pub id_reference_count: usize,
    pub undefined_ref_count: usize,
    pub unused_binding_count: usize,
    pub type_export_count: usize,
    pub invalid_export_count: usize,
    pub import_statement_count: usize,
    pub re_export_count: usize,
    pub binding_span_count: usize,
    pub has_multiple_roots: bool,
    pub uses_attrs: bool,
    pub binds_attrs_explicitly: bool,
    pub inherit_attrs_disabled: bool,
}

impl CroquisSemanticSummary {
    /// Whether fallthrough attributes may be dropped by a fragment root.
    #[inline]
    pub const fn may_lose_fallthrough_attrs(self) -> bool {
        self.has_multiple_roots && !self.binds_attrs_explicitly
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CroquisSemanticSnapshot {
    /// Stable owning source identity supplied by the compilation frontend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_anchor: Option<SourceAnchor>,
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
    pub const fn new(start: u32, end: u32) -> Self {
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
    pub name_is_dynamic: bool,
    pub value: Option<CompactString>,
    pub range: SemanticSourceRange,
    pub dynamic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticEventListenerSnapshot {
    pub name: CompactString,
    pub name_is_dynamic: bool,
    pub handler: Option<CompactString>,
    pub modifiers: Vec<CompactString>,
    pub range: SemanticSourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SemanticSlotUsageSnapshot {
    pub name: CompactString,
    pub name_is_dynamic: bool,
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
