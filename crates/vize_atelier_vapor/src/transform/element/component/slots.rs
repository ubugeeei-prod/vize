//! Resolve named-slot identifiers for components, preserving static
//! `.modifier` segments so dotted slot names stay distinct.

use vize_atelier_core::{DirectiveNode, ExpressionNode};
use vize_carton::String;

/// Resolve a `<template #name.modifiers>` directive into its slot name and
/// whether that name is statically known.
///
/// Static `.modifier` segments are appended so `#item.alpha` and `#item.beta`
/// stay distinct instead of both collapsing onto the bare `item` argument.
pub(super) fn resolve_named_slot(dir: &DirectiveNode<'_>) -> (String, bool) {
    match dir.arg.as_ref() {
        Some(ExpressionNode::Simple(exp)) if exp.is_static => (
            static_slot_name_with_modifiers(exp.content.clone(), dir),
            true,
        ),
        Some(ExpressionNode::Simple(exp)) => (exp.content.clone(), exp.is_static),
        _ => (String::from("default"), true),
    }
}

fn static_slot_name_with_modifiers(mut name: String, dir: &DirectiveNode<'_>) -> String {
    for modifier in dir.modifiers.iter() {
        name.push('.');
        name.push_str(modifier.content.as_str());
    }
    name
}
