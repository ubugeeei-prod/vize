//! Stable semantic snapshot facade for downstream consumers.
//!
//! `Croquis` intentionally stores rich analyzer internals. This module projects
//! those facts into deterministic, serializable view-models that lint, LSP,
//! report, and cross-file crates can share without re-walking parser data.

use super::{
    ComponentUsage, Croquis, CroquisSemanticSummary, EventListener, PassedProp, SlotUsage,
    TemplateExpression, TemplateExpressionKind,
};
use crate::provide::{InjectEntry, InjectPattern, ProvideEntry, ProvideKey};
use crate::reactivity::{ReactiveKind, ReactiveSource, ReactivityLoss, ReactivityLossKind};
use crate::scope::ScopeBinding;
use serde::Serialize;
use vize_carton::{CompactString, String, appends};
use vize_relief::BindingType;

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

impl CroquisSemanticSnapshot {
    /// Build a deterministic snapshot from the current croquis facts.
    pub fn from_croquis(croquis: &Croquis) -> Self {
        Self {
            summary: croquis.semantic_summary(),
            bindings: binding_snapshots(croquis),
            scopes: scope_snapshots(croquis),
            template_expressions: template_expression_snapshots(croquis),
            component_usages: component_usage_snapshots(croquis),
            provides: provide_snapshots(croquis),
            injects: inject_snapshots(croquis),
            reactive_sources: reactive_source_snapshots(croquis),
            reactivity_losses: reactivity_loss_snapshots(croquis),
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

impl SemanticSourceRange {
    #[inline]
    const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

fn binding_snapshots(croquis: &Croquis) -> Vec<SemanticBindingSnapshot> {
    let mut bindings: Vec<_> = croquis
        .bindings
        .iter()
        .map(|(name, kind)| {
            let prop_name = croquis.bindings.props_aliases.get(name).cloned();
            let range = croquis
                .binding_spans
                .get(name)
                .map(|(start, end)| SemanticSourceRange::new(*start, *end));
            SemanticBindingSnapshot {
                id: semantic_id("binding", name, range.map(|range| range.start).unwrap_or(0)),
                name: CompactString::new(name),
                kind: binding_kind(kind),
                category: binding_category(kind),
                prop_name,
                needs_value_in_script: croquis.needs_value_in_script(name),
                range,
            }
        })
        .collect();
    bindings.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
    bindings
}

fn scope_snapshots(croquis: &Croquis) -> Vec<SemanticScopeSnapshot> {
    croquis
        .scopes
        .iter()
        .map(|scope| {
            let mut bindings: Vec<_> = scope
                .bindings()
                .map(|(name, binding)| scope_binding_snapshot(name, binding))
                .collect();
            bindings.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));

            SemanticScopeSnapshot {
                id: scope.id.as_u32(),
                parent_ids: scope.parents.iter().map(|parent| parent.as_u32()).collect(),
                kind: scope.kind.to_display(),
                range: SemanticSourceRange::new(scope.span.start, scope.span.end),
                binding_count: scope.binding_count(),
                bindings,
            }
        })
        .collect()
}

fn scope_binding_snapshot(name: &str, binding: &ScopeBinding) -> SemanticScopeBindingSnapshot {
    SemanticScopeBindingSnapshot {
        name: CompactString::new(name),
        kind: binding_kind(binding.binding_type),
        declaration_offset: binding.declaration_offset,
        used: binding.is_used(),
        mutated: binding.is_mutated(),
    }
}

fn template_expression_snapshots(croquis: &Croquis) -> Vec<SemanticTemplateExpressionSnapshot> {
    let mut expressions: Vec<_> = croquis
        .template_expressions
        .iter()
        .map(template_expression_snapshot)
        .collect();
    expressions.sort_by(|left, right| {
        (left.range.start, left.range.end, left.kind).cmp(&(
            right.range.start,
            right.range.end,
            right.kind,
        ))
    });
    expressions
}

fn template_expression_snapshot(
    expression: &TemplateExpression,
) -> SemanticTemplateExpressionSnapshot {
    SemanticTemplateExpressionSnapshot {
        id: semantic_id("template", expression.kind.as_str(), expression.start),
        content: expression.content.clone(),
        kind: template_expression_kind(expression.kind),
        range: SemanticSourceRange::new(expression.start, expression.end),
        scope_id: expression.scope_id.as_u32(),
        vif_guard: expression.vif_guard.clone(),
    }
}

fn component_usage_snapshots(croquis: &Croquis) -> Vec<SemanticComponentUsageSnapshot> {
    let mut usages: Vec<_> = croquis
        .component_usages
        .iter()
        .map(component_usage_snapshot)
        .collect();
    usages.sort_by(|left, right| {
        (left.range.start, left.range.end, left.name.as_str()).cmp(&(
            right.range.start,
            right.range.end,
            right.name.as_str(),
        ))
    });
    usages
}

fn component_usage_snapshot(usage: &ComponentUsage) -> SemanticComponentUsageSnapshot {
    SemanticComponentUsageSnapshot {
        id: semantic_id("component", usage.name.as_str(), usage.start),
        name: usage.name.clone(),
        range: SemanticSourceRange::new(usage.start, usage.end),
        scope_id: usage.scope_id.as_u32(),
        vif_guard: usage.vif_guard.clone(),
        has_spread_attrs: usage.has_spread_attrs,
        props: usage.props.iter().map(passed_prop_snapshot).collect(),
        events: usage.events.iter().map(event_listener_snapshot).collect(),
        slots: usage.slots.iter().map(slot_usage_snapshot).collect(),
    }
}

fn passed_prop_snapshot(prop: &PassedProp) -> SemanticPassedPropSnapshot {
    SemanticPassedPropSnapshot {
        name: prop.name.clone(),
        value: prop.value.clone(),
        range: SemanticSourceRange::new(prop.start, prop.end),
        dynamic: prop.is_dynamic,
    }
}

fn event_listener_snapshot(event: &EventListener) -> SemanticEventListenerSnapshot {
    SemanticEventListenerSnapshot {
        name: event.name.clone(),
        handler: event.handler.clone(),
        modifiers: event.modifiers.iter().cloned().collect(),
        range: SemanticSourceRange::new(event.start, event.end),
    }
}

fn slot_usage_snapshot(slot: &SlotUsage) -> SemanticSlotUsageSnapshot {
    SemanticSlotUsageSnapshot {
        name: slot.name.clone(),
        scope_vars: slot.scope_vars.iter().cloned().collect(),
        range: SemanticSourceRange::new(slot.start, slot.end),
        scoped: slot.has_scope,
    }
}

fn provide_snapshots(croquis: &Croquis) -> Vec<SemanticProvideSnapshot> {
    let mut provides: Vec<_> = croquis
        .provide_inject
        .provides()
        .iter()
        .map(provide_snapshot)
        .collect();
    provides.sort_by(|left, right| {
        (left.range.start, left.range.end, left.key.as_str()).cmp(&(
            right.range.start,
            right.range.end,
            right.key.as_str(),
        ))
    });
    provides
}

fn provide_snapshot(provide: &ProvideEntry) -> SemanticProvideSnapshot {
    let key = provide_key_value(&provide.key);
    SemanticProvideSnapshot {
        id: semantic_id("provide", key.as_str(), provide.start),
        key,
        key_kind: provide_key_kind(&provide.key),
        value: provide.value.clone(),
        value_type: provide.value_type.clone(),
        from_composable: provide.from_composable.clone(),
        range: SemanticSourceRange::new(provide.start, provide.end),
    }
}

fn inject_snapshots(croquis: &Croquis) -> Vec<SemanticInjectSnapshot> {
    let mut injects: Vec<_> = croquis
        .provide_inject
        .injects()
        .iter()
        .map(inject_snapshot)
        .collect();
    injects.sort_by(|left, right| {
        (left.range.start, left.range.end, left.key.as_str()).cmp(&(
            right.range.start,
            right.range.end,
            right.key.as_str(),
        ))
    });
    injects
}

fn inject_snapshot(inject: &InjectEntry) -> SemanticInjectSnapshot {
    let key = provide_key_value(&inject.key);
    SemanticInjectSnapshot {
        id: semantic_id("inject", key.as_str(), inject.start),
        key,
        key_kind: provide_key_kind(&inject.key),
        local_name: inject.local_name.clone(),
        default_value: inject.default_value.clone(),
        expected_type: inject.expected_type.clone(),
        pattern: inject_pattern_kind(&inject.pattern),
        destructured_names: inject_pattern_names(&inject.pattern),
        from_composable: inject.from_composable.clone(),
        range: SemanticSourceRange::new(inject.start, inject.end),
    }
}

fn reactive_source_snapshots(croquis: &Croquis) -> Vec<SemanticReactiveSourceSnapshot> {
    let mut sources: Vec<_> = croquis
        .reactivity
        .sources()
        .iter()
        .map(reactive_source_snapshot)
        .collect();
    sources.sort_by(|left, right| {
        (left.declaration_offset, left.name.as_str())
            .cmp(&(right.declaration_offset, right.name.as_str()))
    });
    sources
}

fn reactive_source_snapshot(source: &ReactiveSource) -> SemanticReactiveSourceSnapshot {
    SemanticReactiveSourceSnapshot {
        id: semantic_id("reactive", source.name.as_str(), source.declaration_offset),
        name: source.name.clone(),
        kind: reactive_kind_name(source.kind),
        category: reactive_kind_category(source.kind),
        needs_value_access: source.kind.needs_value_access(),
        declaration_offset: source.declaration_offset,
    }
}

fn reactivity_loss_snapshots(croquis: &Croquis) -> Vec<SemanticReactivityLossSnapshot> {
    let mut losses: Vec<_> = croquis
        .reactivity
        .losses()
        .iter()
        .map(reactivity_loss_snapshot)
        .collect();
    losses.sort_by(|left, right| {
        (left.range.start, left.range.end, left.kind).cmp(&(
            right.range.start,
            right.range.end,
            right.kind,
        ))
    });
    losses
}

fn reactivity_loss_snapshot(loss: &ReactivityLoss) -> SemanticReactivityLossSnapshot {
    let mut snapshot = SemanticReactivityLossSnapshot {
        id: semantic_id(
            "reactivity-loss",
            reactivity_loss_kind_name(&loss.kind),
            loss.start,
        ),
        kind: reactivity_loss_kind_name(&loss.kind),
        category: "loss",
        source_name: None,
        target_name: None,
        property_name: None,
        extracted_names: Vec::new(),
        range: SemanticSourceRange::new(loss.start, loss.end),
    };

    match &loss.kind {
        ReactivityLossKind::ReactiveDestructure {
            source_name,
            destructured_props,
        }
        | ReactivityLossKind::RefValueDestructure {
            source_name,
            destructured_props,
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.extracted_names = destructured_props.clone();
        }
        ReactivityLossKind::RefValueExtract {
            source_name,
            target_name,
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.target_name = Some(target_name.clone());
        }
        ReactivityLossKind::ReactivePropertyExtract {
            source_name,
            prop_name,
            target_name,
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.property_name = Some(prop_name.clone());
            snapshot.target_name = Some(target_name.clone());
        }
        ReactivityLossKind::PropsDestructure { destructured_props } => {
            snapshot.extracted_names = destructured_props.clone();
        }
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name: _,
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.target_name = Some(argument_name.clone());
        }
        ReactivityLossKind::GetterCallExtract {
            source_name,
            getter_name,
            target_name,
            ..
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.property_name = Some(getter_name.clone());
            snapshot.target_name = Some(target_name.clone());
        }
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } => {
            snapshot.source_name = Some(source_name.clone());
            snapshot.property_name = Some(alias_name.clone());
            snapshot.target_name = Some(target_name.clone());
        }
        ReactivityLossKind::ReactiveSpread { source_name }
        | ReactivityLossKind::ReactiveReassign { source_name } => {
            snapshot.source_name = Some(source_name.clone());
        }
    }

    snapshot
}

fn semantic_id(kind: &str, name: &str, offset: u32) -> CompactString {
    let mut id = String::default();
    appends!(id, kind, #':', name, #'@', @offset);
    CompactString::new(id.as_str())
}

fn provide_key_value(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(value) | ProvideKey::Symbol(value) => value.clone(),
    }
}

fn provide_key_kind(key: &ProvideKey) -> &'static str {
    match key {
        ProvideKey::String(_) => "string",
        ProvideKey::Symbol(_) => "symbol",
    }
}

fn inject_pattern_kind(pattern: &InjectPattern) -> &'static str {
    match pattern {
        InjectPattern::Simple => "simple",
        InjectPattern::ObjectDestructure(_) => "objectDestructure",
        InjectPattern::ArrayDestructure(_) => "arrayDestructure",
        InjectPattern::IndirectDestructure { .. } => "indirectDestructure",
    }
}

fn inject_pattern_names(pattern: &InjectPattern) -> Vec<CompactString> {
    match pattern {
        InjectPattern::Simple => Vec::new(),
        InjectPattern::ObjectDestructure(names) | InjectPattern::ArrayDestructure(names) => {
            names.clone()
        }
        InjectPattern::IndirectDestructure { props, .. } => props.clone(),
    }
}

fn binding_kind(kind: BindingType) -> &'static str {
    match kind {
        BindingType::SetupLet => "setupLet",
        BindingType::SetupMaybeRef => "setupMaybeRef",
        BindingType::SetupRef => "setupRef",
        BindingType::SetupReactiveConst => "setupReactiveConst",
        BindingType::SetupConst => "setupConst",
        BindingType::Props => "props",
        BindingType::PropsAliased => "propsAliased",
        BindingType::Data => "data",
        BindingType::Options => "options",
        BindingType::LiteralConst => "literalConst",
        BindingType::JsGlobalUniversal => "jsGlobalUniversal",
        BindingType::JsGlobalBrowser => "jsGlobalBrowser",
        BindingType::JsGlobalNode => "jsGlobalNode",
        BindingType::JsGlobalDeno => "jsGlobalDeno",
        BindingType::JsGlobalBun => "jsGlobalBun",
        BindingType::VueGlobal => "vueGlobal",
        BindingType::ExternalModule => "externalModule",
    }
}

fn binding_category(kind: BindingType) -> &'static str {
    match kind {
        BindingType::SetupLet
        | BindingType::SetupMaybeRef
        | BindingType::SetupRef
        | BindingType::SetupReactiveConst
        | BindingType::SetupConst
        | BindingType::LiteralConst => "setup",
        BindingType::Props | BindingType::PropsAliased => "props",
        BindingType::Data => "data",
        BindingType::Options => "options",
        BindingType::JsGlobalUniversal
        | BindingType::JsGlobalBrowser
        | BindingType::JsGlobalNode
        | BindingType::JsGlobalDeno
        | BindingType::JsGlobalBun => "jsGlobal",
        BindingType::VueGlobal => "vueGlobal",
        BindingType::ExternalModule => "externalModule",
    }
}

fn template_expression_kind(kind: TemplateExpressionKind) -> &'static str {
    match kind {
        TemplateExpressionKind::Interpolation => "interpolation",
        TemplateExpressionKind::VBind => "vBind",
        TemplateExpressionKind::VOn => "vOn",
        TemplateExpressionKind::VIf => "vIf",
        TemplateExpressionKind::VShow => "vShow",
        TemplateExpressionKind::VModel => "vModel",
    }
}

fn reactive_kind_name(kind: ReactiveKind) -> &'static str {
    match kind {
        ReactiveKind::Ref => "ref",
        ReactiveKind::ShallowRef => "shallowRef",
        ReactiveKind::Reactive => "reactive",
        ReactiveKind::ShallowReactive => "shallowReactive",
        ReactiveKind::Computed => "computed",
        ReactiveKind::Readonly => "readonly",
        ReactiveKind::ShallowReadonly => "shallowReadonly",
        ReactiveKind::ToRef => "toRef",
        ReactiveKind::ToRefs => "toRefs",
    }
}

fn reactive_kind_category(kind: ReactiveKind) -> &'static str {
    match kind {
        ReactiveKind::Ref
        | ReactiveKind::ShallowRef
        | ReactiveKind::ToRef
        | ReactiveKind::ToRefs => "ref",
        ReactiveKind::Reactive | ReactiveKind::ShallowReactive => "reactive",
        ReactiveKind::Computed => "computed",
        ReactiveKind::Readonly | ReactiveKind::ShallowReadonly => "readonly",
    }
}

fn reactivity_loss_kind_name(kind: &ReactivityLossKind) -> &'static str {
    match kind {
        ReactivityLossKind::ReactiveDestructure { .. } => "reactiveDestructure",
        ReactivityLossKind::RefValueDestructure { .. } => "refValueDestructure",
        ReactivityLossKind::RefValueExtract { .. } => "refValueExtract",
        ReactivityLossKind::ReactivePropertyExtract { .. } => "reactivePropertyExtract",
        ReactivityLossKind::PropsDestructure { .. } => "propsDestructure",
        ReactivityLossKind::FunctionArgumentExtract { .. } => "functionArgumentExtract",
        ReactivityLossKind::GetterCallExtract { .. } => "getterCallExtract",
        ReactivityLossKind::PlainValueAlias { .. } => "plainValueAlias",
        ReactivityLossKind::ReactiveSpread { .. } => "reactiveSpread",
        ReactivityLossKind::ReactiveReassign { .. } => "reactiveReassign",
    }
}
