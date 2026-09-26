//! Slot content: `v-slot` / `#name[="params"]` on a `<template>` or on its
//! component. The name is static or computed and the parameter
//! pattern is one the shared generator scopes exactly.

use vize_atelier_core::steps::expression::is_template_global;
use vize_carton::Vec;
use vize_s3::{
    op::OpId,
    operand::{Operand, OperandRole as Role, ValueKind},
};

use super::super::{Binding, BindingKind, Expr, Node};
use super::{
    Result,
    component::component_prop,
    operands::{js, one},
};
use crate::s3::{LegacyReason, retained::Retained};

pub(super) fn slot<'a>(
    values: &[Operand<'a>],
    retained: &Retained<'_, 'a>,
) -> Result<(OpId, Binding<'a>)> {
    let kind = one(values, Role::BindingKind)?;
    let name = one(values, Role::Name)?;
    let params = one(values, Role::Params)?;
    let target = kind.target.ok_or(LegacyReason::Structure)?;
    if kind.value.kind != ValueKind::Literal
        || kind.value.text != "slot-content"
        || values.len() != 4
        || one(values, Role::Value)?.value.kind != ValueKind::Absent
        || values
            .iter()
            .any(|v| v.target != Some(target) || v.region.is_some() || v.name.is_some())
    {
        return Err(LegacyReason::Component.into());
    }
    let spans = [
        (name.value.span.start, name.value.span.end),
        (params.value.span.start, params.value.span.end),
    ];
    let (name, dynamic_name) = match name.value.kind {
        ValueKind::Absent => ("default", None),
        ValueKind::Literal if component_prop(name.value.text) => (name.value.text, None),
        ValueKind::Js => (name.value.text, Some(js(retained, name)?)),
        _ => return Err(LegacyReason::Component.into()),
    };
    let params = match params.value.kind {
        ValueKind::Absent => "",
        ValueKind::Js if pattern(params.value.text) => params.value.text.trim(),
        _ => return Err(LegacyReason::Component.into()),
    };
    Ok((
        target,
        Binding {
            kind: BindingKind::Slot,
            name,
            dynamic_name,
            value: Expr::plain(params),
            modifiers: Vec::new_in(&retained.allocator()),
            merge: None,
            model_element: None,
            position: 0,
            spans,
        },
    ))
}

/// A plain identifier, or a flat object pattern of distinct identifiers.
/// Renames, defaults, rest and nested patterns stay legacy.
fn pattern(text: &str) -> bool {
    let identifier = |name: &str| {
        name.starts_with(|c: char| c.is_ascii_alphabetic())
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !super::ident::reserved_keyword(name)
            && !is_template_global(name)
    };
    let text = text.trim();
    match text
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
    {
        Some(fields) => {
            let names: std::vec::Vec<&str> = fields.split(',').map(str::trim).collect();
            names.iter().all(|name| identifier(name)) && !has_repeat(&names)
        }
        None => identifier(text),
    }
}

/// Whether some item equals an earlier one.
fn has_repeat<T: PartialEq>(items: &[T]) -> bool {
    (items.iter().enumerate()).any(|(at, item)| items.iter().take(at).any(|seen| seen == item))
}

/// The slot binding's name, when `node` carries one.
pub(super) fn slot_name<'a>(node: &Node<'a>) -> Option<&'a str> {
    node.bindings
        .iter()
        .find(|binding| binding.kind == BindingKind::Slot)
        .filter(|binding| binding.dynamic_name.is_none())
        .map(|binding| binding.name)
}

pub(super) fn has_slot(node: &Node<'_>) -> bool {
    node.bindings
        .iter()
        .any(|binding| binding.kind == BindingKind::Slot)
}

/// Direct named templates and checked control carriers belong to a component.
pub(super) fn check(nodes: &[Node<'_>], parents: &[Option<usize>]) -> Result<()> {
    super::structural_slots::check(nodes, parents)
}
