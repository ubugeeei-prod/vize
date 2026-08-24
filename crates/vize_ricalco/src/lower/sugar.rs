//! Vue 2 `slot-scope` / `scope` attribute capture at lowering.
//!
//! The shipped pre-transform (`desugar_scoped_slot_attrs`) rewrites
//! these into a `v-slot` directive before the rest of the lane runs.
//! S2 keeps the authored sugar as [`BindingOp::VueSlotScope`] so the
//! legalizing pass can turn it into `ui.slot-content` 1:1 (same
//! page-order id, same scope-fact key). A carrier that already authors
//! `v-slot` is left alone — the shipped lane will not emit a
//! conflicting directive.

use alloc::vec::Vec as StdVec;

use vize_s0::{Box, String, cstr};
use vize_s1::Element;

use vize_s2::op::{BindingOp, VueSlotScopeOp};
use vize_s2::scope::{ScopeBinding, ScopeFacts, ScopeOrigin};

use super::cx::{Cx, attr_slice, attr_span};
use super::directive::{AttrForm, Head};
use super::element::{Analyzed, attr_value_text};
use super::expr::{expr_at, simple_identifier};

/// Whether this element should consume `slot-scope`/`scope` (and the
/// companion static `slot`) into a dialect binding.
pub(crate) fn should_take(cx: &Cx<'_>, element: &Element<'_>, analyzed: &Analyzed<'_>) -> bool {
    cx.caps.scoped_slot_attrs
        && !has_v_slot(analyzed)
        && scope_attr_index(element, analyzed).is_some()
}

/// The first static `slot-scope` or (on `<template>` only) `scope`
/// attribute, in authored order. Vue 2.5+'s `slot-scope` is legal on
/// any element; `scope` (2.1) was the `<template>`-only precursor, so
/// `<div scope="props">` stays an ordinary HTML attribute.
pub(crate) fn scope_attr_index(element: &Element<'_>, analyzed: &Analyzed<'_>) -> Option<usize> {
    let on_template = element.tag().eq_ignore_ascii_case("template");
    element
        .open
        .attrs
        .iter()
        .enumerate()
        .position(|(index, attr)| {
            matches!(analyzed.forms.get(index), Some(AttrForm::Static))
                && (attr.name.text == "slot-scope" || (attr.name.text == "scope" && on_template))
        })
}

/// The companion static `slot="name"` attribute, consumed as the
/// dialect op's slot name. Dynamic `:slot` stays a bind.
pub(crate) fn companion_slot_index(
    element: &Element<'_>,
    analyzed: &Analyzed<'_>,
) -> Option<usize> {
    element
        .open
        .attrs
        .iter()
        .enumerate()
        .position(|(index, attr)| {
            matches!(analyzed.forms.get(index), Some(AttrForm::Static)) && attr.name.text == "slot"
        })
}

/// Lower the scope attribute into `vue.slot-scope`. Scope facts key
/// this binding so the 1:1 rewrite to `ui.slot-content` keeps the same
/// side-table entry.
pub(crate) fn lower_slot_scope<'a>(
    cx: &mut Cx<'a>,
    element: &Element<'a>,
    index: usize,
    companion_slot: Option<usize>,
) -> BindingOp<'a> {
    let attr = &element.open.attrs[index];
    let span = attr_span(cx, attr);
    let node = cx.mint_op();
    let name = companion_slot.and_then(|slot_index| {
        attr_value_text(element, slot_index)
            .map(str::trim)
            .filter(|text| !text.is_empty())
    });
    let params = attr_value_text(element, index)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(|text| expr_at(cx, text));
    if let Some(expr) = &params {
        let tag = cx.mint_scope();
        let mut scope_bindings = StdVec::new();
        if let Some(bound) = simple_identifier(expr) {
            scope_bindings.push(ScopeBinding {
                name: String::from(bound),
                origin: ScopeOrigin::Authored { span: expr.span() },
            });
        }
        cx.attach_scope(
            node,
            ScopeFacts {
                tag,
                bindings: scope_bindings,
            },
        );
    }
    let after = match name {
        None => String::from("vue.slot-scope"),
        Some(text) => cstr!("vue.slot-scope \"{text}\""),
    };
    cx.record("lower.slot-scope", node, attr_slice(cx, attr), after, span);
    BindingOp::VueSlotScope(Box::new_in(
        VueSlotScopeOp { name, params, span },
        &cx.allocator,
    ))
}

fn has_v_slot(analyzed: &Analyzed<'_>) -> bool {
    analyzed
        .forms
        .iter()
        .any(|form| matches!(form, AttrForm::Directive(directive) if directive.head == Head::Slot))
}
