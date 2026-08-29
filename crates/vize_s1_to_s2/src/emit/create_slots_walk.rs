//! Walk helpers for `createSlots` entry collection.

use vize_s2::op::{BindingOp, ElementOp, ForOp, IfBranch, IfOp, Op, Region, SlotContentOp};

use super::EmitCx;
use super::slots::is_whitespace_text;

pub(super) fn slot_content<'a>(element: &'a ElementOp<'a>) -> Option<&'a SlotContentOp<'a>> {
    element.bindings.iter().find_map(|binding| match binding {
        BindingOp::SlotContent(content) => Some(&**content),
        _ => None,
    })
}

pub(super) fn slot_template_content<'a>(
    element: &'a ElementOp<'a>,
) -> Option<&'a SlotContentOp<'a>> {
    if element.tag == "template" {
        slot_content(element)
    } else {
        None
    }
}

pub(super) fn first_slot_template<'a>(
    region: &'a Region<'a>,
) -> Option<(usize, &'a ElementOp<'a>, &'a SlotContentOp<'a>)> {
    region.ops.iter().enumerate().find_map(|(i, op)| match op {
        Op::Element(element) => {
            slot_template_content(element).map(|content| (i, &**element, content))
        }
        _ => None,
    })
}

pub(super) fn is_slot_if(if_op: &IfOp<'_>) -> bool {
    if_op.branches.iter().any(is_slot_if_branch)
}

pub(super) fn is_slot_for(for_op: &ForOp<'_>) -> bool {
    is_slot_template_carrier(&for_op.region)
}

/// `createSlots` owns `v-if` / `v-for` *on* a slot template. An unwrapped
/// `<template v-if>` / `<template v-for>` that merely *contains* a nested
/// `#slot` plus other children must stay on the default-slot path — otherwise
/// `skip_ops` drops the siblings.
fn is_slot_if_branch(branch: &IfBranch<'_>) -> bool {
    is_slot_template_carrier(&branch.region)
}

fn is_slot_template_carrier(region: &Region<'_>) -> bool {
    first_slot_template(region).is_some() && !region_has_non_slot_content(region)
}

fn region_has_non_slot_content(region: &Region<'_>) -> bool {
    region.ops.iter().any(|op| {
        !is_whitespace_text(op)
            && !matches!(
                op,
                Op::Element(element) if slot_template_content(element).is_some()
            )
    })
}

pub(super) fn skip_ops(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) {
    for op in ops {
        skip_op(cx, op);
    }
}

fn skip_op(cx: &mut EmitCx<'_>, op: &Op<'_>) {
    let _id = cx.walk.mint();
    match op {
        Op::Element(element) => {
            cx.walk.skip(element.bindings.len());
            skip_ops(cx, &element.children.ops);
        }
        Op::Component(component) => {
            cx.walk.skip(component.bindings.len());
            skip_ops(cx, &component.children.ops);
        }
        Op::If(if_op) => {
            for branch in if_op.branches.iter() {
                skip_ops(cx, &branch.region.ops);
            }
        }
        Op::For(for_op) => skip_ops(cx, &for_op.region.ops),
        Op::Slot(slot) => {
            cx.walk.skip(slot.bindings.len());
            skip_ops(cx, &slot.fallback.ops);
        }
        Op::Text(_) | Op::Interpolation(_) => {}
    }
}
