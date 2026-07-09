use super::super::{
    ComponentUsage, Croquis, EventListener, PassedProp, SlotUsage, TemplateExpression,
};
use super::names::*;
use super::types::*;
use crate::provide::{InjectEntry, ProvideEntry};
use crate::reactivity::{ReactiveSource, ReactivityLoss, ReactivityLossKind};
use crate::scope::ScopeBinding;
use vize_carton::CompactString;

pub(super) fn binding_snapshots(croquis: &Croquis) -> Vec<SemanticBindingSnapshot> {
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

pub(super) fn scope_snapshots(croquis: &Croquis) -> Vec<SemanticScopeSnapshot> {
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

pub(super) fn template_expression_snapshots(
    croquis: &Croquis,
) -> Vec<SemanticTemplateExpressionSnapshot> {
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

pub(super) fn component_usage_snapshots(croquis: &Croquis) -> Vec<SemanticComponentUsageSnapshot> {
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

pub(super) fn provide_snapshots(croquis: &Croquis) -> Vec<SemanticProvideSnapshot> {
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

pub(super) fn inject_snapshots(croquis: &Croquis) -> Vec<SemanticInjectSnapshot> {
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

pub(super) fn reactive_source_snapshots(croquis: &Croquis) -> Vec<SemanticReactiveSourceSnapshot> {
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

pub(super) fn reactivity_loss_snapshots(croquis: &Croquis) -> Vec<SemanticReactivityLossSnapshot> {
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
