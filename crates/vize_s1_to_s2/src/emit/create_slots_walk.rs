//! Walk helpers for `createSlots` entry collection.

use vize_davinci::id::NodeId;
use vize_s0::ensure_sufficient_stack;
use vize_s2::op::{BindingOp, ElementOp, ForOp, IfBranch, IfOp, Op, Region, SlotContentOp};

use super::EmitCx;
use super::slots::is_whitespace_text;
use crate::pass::walk::PageWalk;

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

pub(super) fn is_slot_if(cx: &EmitCx<'_>, id: Option<NodeId>, if_op: &IfOp<'_>) -> bool {
    if_op
        .branches
        .iter()
        .enumerate()
        .any(|(branch_index, branch)| is_slot_if_branch(cx, id, branch_index, branch))
}

pub(super) fn is_slot_for(cx: &EmitCx<'_>, id: Option<NodeId>, for_op: &ForOp<'_>) -> bool {
    id.and_then(|id| cx.for_wrappers.get(id)).is_none() && is_slot_template_carrier(&for_op.region)
}

pub(super) fn advance_after_op(walk: &mut PageWalk, op: &Op<'_>) {
    match op {
        Op::Element(element) => {
            walk.skip(element.bindings.len());
            advance_after_ops(walk, &element.children.ops);
        }
        Op::Component(component) => {
            walk.skip(component.bindings.len());
            advance_after_ops(walk, &component.children.ops);
        }
        Op::If(if_op) => {
            for branch in if_op.branches.iter() {
                advance_after_ops(walk, &branch.region.ops);
            }
        }
        Op::For(for_op) => advance_after_ops(walk, &for_op.region.ops),
        Op::Slot(slot) => {
            walk.skip(slot.bindings.len());
            advance_after_ops(walk, &slot.fallback.ops);
        }
        Op::Text(_) | Op::Interpolation(_) => {}
    }
}

fn advance_after_ops(walk: &mut PageWalk, ops: &[Op<'_>]) {
    ensure_sufficient_stack(|| advance_after_ops_guarded(walk, ops));
}

fn advance_after_ops_guarded(walk: &mut PageWalk, ops: &[Op<'_>]) {
    for op in ops {
        let _id = walk.mint();
        advance_after_op(walk, op);
    }
}

/// `createSlots` owns `v-if` / `v-for` *on* a slot template. An unwrapped
/// `<template v-if>` / `<template v-for>` that merely *contains* nested
/// `#slot` children must stay on the default-slot path — otherwise
/// `skip_ops` keeps only `first_slot_template` and drops the rest.
fn is_slot_if_branch(
    cx: &EmitCx<'_>,
    id: Option<NodeId>,
    branch_index: usize,
    branch: &IfBranch<'_>,
) -> bool {
    !id.and_then(|id| cx.wrappers.get(id))
        .and_then(|keys| keys.from_template.get(branch_index).copied())
        .unwrap_or(false)
        && is_slot_template_carrier(&branch.region)
}

fn is_slot_template_carrier(region: &Region<'_>) -> bool {
    let mut slot_templates = 0usize;
    for op in region.ops.iter() {
        if is_whitespace_text(op) {
            continue;
        }
        if matches!(op, Op::Element(element) if slot_template_content(element).is_some()) {
            slot_templates += 1;
        } else {
            return false;
        }
    }
    slot_templates == 1
}

pub(super) fn skip_ops(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) {
    ensure_sufficient_stack(|| skip_ops_guarded(cx, ops));
}

fn skip_ops_guarded(cx: &mut EmitCx<'_>, ops: &[Op<'_>]) {
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
