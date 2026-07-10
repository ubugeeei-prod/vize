use super::super::{
    ComponentUsage, Croquis, EventListener, PassedProp, SlotUsage, TemplateExpression,
};
use super::{names, types as snapshot};
use crate::provide::{InjectEntry, ProvideEntry};
use crate::reactivity::ReactiveSource;
use crate::scope::ScopeBinding;
use vize_carton::CompactString;

pub(super) fn binding_snapshots(croquis: &Croquis) -> Vec<snapshot::SemanticBindingSnapshot> {
    let mut bindings: Vec<_> = croquis
        .bindings
        .iter()
        .map(|(name, kind)| {
            let prop_name = croquis.bindings.props_aliases.get(name).cloned();
            let range = croquis
                .binding_spans
                .get(name)
                .map(|(start, end)| snapshot::SemanticSourceRange::new(*start, *end));
            snapshot::SemanticBindingSnapshot {
                id: names::semantic_id(
                    "binding",
                    name,
                    range.map(|range| range.start).unwrap_or(0),
                ),
                name: CompactString::new(name),
                kind: names::binding_kind(kind),
                category: names::binding_category(kind),
                prop_name,
                needs_value_in_script: croquis.needs_value_in_script(name),
                range,
            }
        })
        .collect();
    bindings.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
    bindings
}

pub(super) fn scope_snapshots(croquis: &Croquis) -> Vec<snapshot::SemanticScopeSnapshot> {
    croquis
        .scopes
        .iter()
        .map(|scope| {
            let mut bindings: Vec<_> = scope
                .bindings()
                .map(|(name, binding)| scope_binding_snapshot(name, binding))
                .collect();
            bindings.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));

            snapshot::SemanticScopeSnapshot {
                id: scope.id.as_u32(),
                parent_ids: scope.parents.iter().map(|parent| parent.as_u32()).collect(),
                kind: scope.kind.to_display(),
                range: snapshot::SemanticSourceRange::new(scope.span.start, scope.span.end),
                binding_count: scope.binding_count(),
                bindings,
            }
        })
        .collect()
}

fn scope_binding_snapshot(
    name: &str,
    binding: &ScopeBinding,
) -> snapshot::SemanticScopeBindingSnapshot {
    snapshot::SemanticScopeBindingSnapshot {
        name: CompactString::new(name),
        kind: names::binding_kind(binding.binding_type),
        declaration_offset: binding.declaration_offset,
        used: binding.is_used(),
        mutated: binding.is_mutated(),
    }
}

pub(super) fn template_expression_snapshots(
    croquis: &Croquis,
) -> Vec<snapshot::SemanticTemplateExpressionSnapshot> {
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
) -> snapshot::SemanticTemplateExpressionSnapshot {
    snapshot::SemanticTemplateExpressionSnapshot {
        id: names::semantic_id("template", expression.kind.as_str(), expression.start),
        content: expression.content.clone(),
        kind: names::template_expression_kind(expression.kind),
        range: snapshot::SemanticSourceRange::new(expression.start, expression.end),
        scope_id: expression.scope_id.as_u32(),
        vif_guard: expression.vif_guard.clone(),
    }
}

pub(super) fn component_usage_snapshots(
    croquis: &Croquis,
) -> Vec<snapshot::SemanticComponentUsageSnapshot> {
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

fn component_usage_snapshot(usage: &ComponentUsage) -> snapshot::SemanticComponentUsageSnapshot {
    snapshot::SemanticComponentUsageSnapshot {
        id: names::semantic_id("component", usage.name.as_str(), usage.start),
        name: usage.name.clone(),
        range: snapshot::SemanticSourceRange::new(usage.start, usage.end),
        scope_id: usage.scope_id.as_u32(),
        vif_guard: usage.vif_guard.clone(),
        has_spread_attrs: usage.has_spread_attrs,
        props: usage.props.iter().map(passed_prop_snapshot).collect(),
        events: usage.events.iter().map(event_listener_snapshot).collect(),
        slots: usage.slots.iter().map(slot_usage_snapshot).collect(),
    }
}

fn passed_prop_snapshot(prop: &PassedProp) -> snapshot::SemanticPassedPropSnapshot {
    snapshot::SemanticPassedPropSnapshot {
        name: prop.name.clone(),
        name_is_dynamic: prop.name_is_dynamic,
        value: prop.value.clone(),
        range: snapshot::SemanticSourceRange::new(prop.start, prop.end),
        dynamic: prop.is_dynamic,
    }
}

fn event_listener_snapshot(event: &EventListener) -> snapshot::SemanticEventListenerSnapshot {
    snapshot::SemanticEventListenerSnapshot {
        name: event.name.clone(),
        name_is_dynamic: event.name_is_dynamic,
        handler: event.handler.clone(),
        modifiers: event.modifiers.iter().cloned().collect(),
        range: snapshot::SemanticSourceRange::new(event.start, event.end),
    }
}

fn slot_usage_snapshot(slot: &SlotUsage) -> snapshot::SemanticSlotUsageSnapshot {
    snapshot::SemanticSlotUsageSnapshot {
        name: slot.name.clone(),
        name_is_dynamic: slot.name_is_dynamic,
        scope_vars: slot.scope_vars.iter().cloned().collect(),
        range: snapshot::SemanticSourceRange::new(slot.start, slot.end),
        scoped: slot.has_scope,
    }
}

pub(super) fn provide_snapshots(croquis: &Croquis) -> Vec<snapshot::SemanticProvideSnapshot> {
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

fn provide_snapshot(provide: &ProvideEntry) -> snapshot::SemanticProvideSnapshot {
    let key = names::provide_key_value(&provide.key);
    snapshot::SemanticProvideSnapshot {
        id: names::semantic_id("provide", key.as_str(), provide.start),
        key,
        key_kind: names::provide_key_kind(&provide.key),
        value: provide.value.clone(),
        value_type: provide.value_type.clone(),
        from_composable: provide.from_composable.clone(),
        range: snapshot::SemanticSourceRange::new(provide.start, provide.end),
    }
}

pub(super) fn inject_snapshots(croquis: &Croquis) -> Vec<snapshot::SemanticInjectSnapshot> {
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

fn inject_snapshot(inject: &InjectEntry) -> snapshot::SemanticInjectSnapshot {
    let key = names::provide_key_value(&inject.key);
    snapshot::SemanticInjectSnapshot {
        id: names::semantic_id("inject", key.as_str(), inject.start),
        key,
        key_kind: names::provide_key_kind(&inject.key),
        local_name: inject.local_name.clone(),
        default_value: inject.default_value.clone(),
        expected_type: inject.expected_type.clone(),
        pattern: names::inject_pattern_kind(&inject.pattern),
        destructured_names: names::inject_pattern_names(&inject.pattern),
        from_composable: inject.from_composable.clone(),
        range: snapshot::SemanticSourceRange::new(inject.start, inject.end),
    }
}

pub(super) fn reactive_source_snapshots(
    croquis: &Croquis,
) -> Vec<snapshot::SemanticReactiveSourceSnapshot> {
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

fn reactive_source_snapshot(source: &ReactiveSource) -> snapshot::SemanticReactiveSourceSnapshot {
    snapshot::SemanticReactiveSourceSnapshot {
        id: names::semantic_id("reactive", source.name.as_str(), source.declaration_offset),
        name: source.name.clone(),
        kind: names::reactive_kind_name(source.kind),
        category: names::reactive_kind_category(source.kind),
        needs_value_access: source.kind.needs_value_access(),
        declaration_offset: source.declaration_offset,
    }
}
