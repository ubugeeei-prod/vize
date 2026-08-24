//! Branch-key extraction for the `v-if` pass, split out when the
//! element/binding-family installment (P2-9 series 5) landed the dynamic
//! arm: with `ui.bind` on the surface, a carrier's `:key` binding is
//! extractable exactly where the legacy `extract_key_prop` reads it.
//!
//! # The selection rule, mirrored from the legacy transform
//!
//! `extract_key_prop` scans the carrier's props in authored order and
//! takes the **first** `key` attribute or static-argument `:key`
//! directive. The S2 surface splits attributes from bindings, so
//! authored order is re-derived from spans: the earlier `span.start`
//! wins. A static key **leaves the attribute surface** (both lanes
//! remove it — the legacy prop moves to `user_key`); a dynamic key's
//! binding op **stays** — a pass removing a binding op would shift every
//! page-order id after it — and the fact records which binding carries
//! it ([`super::BranchKeyKind::Dynamic`]`::bind_index`) so realization
//! and the differential projection can exclude it. A dynamic-argument
//! `:[key]` is not a candidate (the legacy arg-content match admits it —
//! a recorded quirk the differential lane counts rather than imitates).

use vize_carton::{Span, String, Vec, cstr};
use vize_disegno::op::{Attribute, BindingOp, DynamicName, Op};

use super::{BranchKey, BranchKeyKind};

/// The attribute/binding surface of a branch's carrier op: the branch
/// region's single root when it is an element, component, or slot
/// outlet **and it is the branch's own carrier** (op span == branch
/// span).
///
/// The span-identity guard is what keeps an unwrapped
/// `<template v-if><div key>` honest: the lowering unwraps the wrapper,
/// so the region's single root can be an inner child whose `key`
/// belongs to that child's own vnode — an inner op's span is strictly
/// inside the branch's, the carrier's is equal. A `ui.for` root means
/// the key belongs to `v-for` (Vue 3 precedence — the legacy transform
/// skips extraction the same way).
fn carrier_surface<'w, 'a>(
    branch_span: Span,
    ops: &'w mut [Op<'a>],
) -> Option<(
    &'w mut Vec<'a, Attribute<'a>>,
    &'w mut Vec<'a, BindingOp<'a>>,
)> {
    let [op] = ops else {
        return None;
    };
    match op {
        Op::Element(element) if element.span == branch_span => {
            let element = &mut **element;
            Some((&mut element.attributes, &mut element.bindings))
        }
        Op::Component(component) if component.span == branch_span => {
            let component = &mut **component;
            Some((&mut component.attributes, &mut component.bindings))
        }
        Op::Slot(slot) if slot.span == branch_span => {
            let slot = &mut **slot;
            Some((&mut slot.attributes, &mut slot.bindings))
        }
        Op::Element(_)
        | Op::Component(_)
        | Op::Slot(_)
        | Op::Text(_)
        | Op::Interpolation(_)
        | Op::If(_)
        | Op::For(_) => None,
    }
}

/// A `ui.bind` whose name position is the static `key`.
fn is_key_bind(binding: &BindingOp<'_>) -> bool {
    match binding {
        BindingOp::Bind(bind) => matches!(bind.name, Some(DynamicName::Static("key"))),
        BindingOp::On(_)
        | BindingOp::Model(_)
        | BindingOp::SlotContent(_)
        | BindingOp::VueDirective(_)
        | BindingOp::VueCssBind(_)
        | BindingOp::VueSync(_)
        | BindingOp::VueSlotScope(_)
        | BindingOp::VueOnce(_)
        | BindingOp::VueMemo(_) => false,
    }
}

/// Extract the branch key off the carrier surface, when one is authored:
/// the first key spelling in authored (span) order.
pub(super) fn take_carrier_key(branch_span: Span, ops: &mut [Op<'_>]) -> Option<BranchKey> {
    let (attributes, bindings) = carrier_surface(branch_span, ops)?;
    let static_at = attributes
        .iter()
        .position(|attribute| attribute.name == "key")
        .map(|index| (attributes[index].span.start, index));
    let dynamic_at = bindings
        .iter()
        .position(is_key_bind)
        .map(|index| (binding_span(&bindings[index]).start, index));
    match (static_at, dynamic_at) {
        (Some((attr_start, index)), Some((bind_start, _))) if attr_start < bind_start => {
            Some(take_static(attributes, index))
        }
        (Some((_, index)), None) => Some(take_static(attributes, index)),
        (_, Some((_, index))) => Some(read_dynamic(bindings, index)),
        (None, None) => None,
    }
}

/// Remove the static `key` attribute (both lanes drop it from the
/// surface) and return it as the fact.
fn take_static<'a>(attributes: &mut Vec<'a, Attribute<'a>>, index: usize) -> BranchKey {
    let attribute = attributes.remove(index);
    BranchKey {
        kind: BranchKeyKind::Static(attribute.value.map(String::from)),
        span: attribute.span,
    }
}

/// Read the `:key` binding as the fact; the op stays on the surface.
fn read_dynamic<'a>(bindings: &[BindingOp<'a>], index: usize) -> BranchKey {
    let BindingOp::Bind(bind) = &bindings[index] else {
        unreachable!("is_key_bind admitted only ui.bind")
    };
    let source = bind
        .value
        .as_ref()
        .map(|value| String::from(value.source()))
        .unwrap_or_default();
    BranchKey {
        kind: BranchKeyKind::Dynamic {
            source,
            bind_index: Some(index),
        },
        span: bind.span,
    }
}

/// The provenance `before` spelling of one extracted key (the arena
/// attribute does not retain its authored quoting, so the spelling is
/// folio-normalized).
pub(super) fn key_spelling(key: &BranchKey) -> String {
    match &key.kind {
        BranchKeyKind::Static(Some(value)) => cstr!("key=\"{value}\""),
        BranchKeyKind::Static(None) => String::from("key"),
        BranchKeyKind::Dynamic { source, .. } => cstr!(":key=\"{source}\""),
    }
}

fn binding_span(binding: &BindingOp<'_>) -> Span {
    match binding {
        BindingOp::Bind(bind) => bind.span,
        BindingOp::On(on) => on.span,
        BindingOp::Model(model) => model.span,
        BindingOp::SlotContent(content) => content.span,
        BindingOp::VueDirective(directive) => directive.span,
        BindingOp::VueCssBind(bind) => bind.span,
        BindingOp::VueSync(sync) => sync.span,
        BindingOp::VueSlotScope(scope) => scope.span,
        BindingOp::VueOnce(once) => once.span,
        BindingOp::VueMemo(memo) => memo.span,
    }
}
