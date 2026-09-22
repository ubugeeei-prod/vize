//! Slot content: `v-slot` / `#name[="params"]` on a `<template>` or on its
//! component. The name is static (absent means `default`) and the parameter
//! pattern is one the shared generator scopes exactly.

use vize_atelier_core::steps::expression::is_template_global;
use vize_carton::{Allocator, Vec};
use vize_s3::{
    op::OpId,
    operand::{Operand, OperandRole as Role, ValueKind},
};

use super::super::{Binding, BindingKind, Content, Expr, Node};
use super::{Result, component::component_prop, operands::one};
use crate::s3::LegacyReason;

pub(super) fn slot<'a>(
    values: &[Operand<'a>],
    alloc: &'a Allocator,
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
    let name = match name.value.kind {
        ValueKind::Absent => "default",
        ValueKind::Literal if component_prop(name.value.text) => name.value.text,
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
            value: Expr::plain(params),
            modifiers: Vec::new_in(&alloc),
            merge: None,
            position: 0,
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
            names.iter().all(|name| identifier(name))
                && (1..names.len()).all(|at| !names[..at].contains(&names[at]))
        }
        None => identifier(text),
    }
}

/// The slot binding's name, when `node` carries one.
fn slot_name<'a>(node: &Node<'a>) -> Option<&'a str> {
    node.bindings
        .iter()
        .find(|binding| binding.kind == BindingKind::Slot)
        .map(|binding| binding.name)
}

/// A `<template>` is slot content only, directly under a component. A
/// component takes either its own `v-slot` or named templates, never both,
/// never implicit content beside templates, and no slot name twice.
pub(super) fn check(nodes: &[Node<'_>], parents: &[Option<usize>]) -> Result<()> {
    for (index, node) in nodes.iter().enumerate() {
        match node.content {
            Content::Element {
                tag: "template", ..
            } => {
                let parent = parents[index].map(|parent| &nodes[parent].content);
                if slot_name(node).is_none() || !matches!(parent, Some(Content::Component { .. })) {
                    return Err(LegacyReason::Element.into());
                }
            }
            Content::Component { .. } => {
                let named: std::vec::Vec<_> = node
                    .children
                    .iter()
                    .filter_map(|child| slot_name(&nodes[*child]))
                    .collect();
                let mixed = !named.is_empty()
                    && (slot_name(node).is_some() || named.len() != node.children.len());
                if mixed || (1..named.len()).any(|at| named[..at].contains(&named[at])) {
                    return Err(LegacyReason::Component.into());
                }
            }
            _ => {}
        }
    }
    Ok(())
}
