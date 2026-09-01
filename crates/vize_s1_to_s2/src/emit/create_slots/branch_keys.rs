use vize_s2::op::{ForOp, IfOp, Op, Region};

use crate::emit::create_slots_walk::{
    advance_after_op, first_slot_template, is_slot_for, is_slot_if, slot_template_content,
};
use crate::emit::slots::is_whitespace_text;
use crate::emit::{EmitCx, EmitError};
use crate::pass::walk::PageWalk;

pub(super) fn is_template_for_slot_outlet_entry(
    cx: &EmitCx<'_>,
    id: Option<vize_davinci::id::NodeId>,
    for_op: &ForOp<'_>,
) -> bool {
    id.and_then(|id| cx.for_wrappers.get(id)).is_some()
        && matches!(for_op.region.ops.as_slice(), [Op::Slot(_)])
}

pub(super) fn peek_id(cx: &EmitCx<'_>) -> Option<vize_davinci::id::NodeId> {
    let mut walk = cx.walk.clone();
    walk.mint()
}

pub(super) fn with_branch_key<T>(
    cx: &mut EmitCx<'_>,
    next_key: &mut u32,
    write: impl FnOnce(&mut EmitCx<'_>) -> Result<T, EmitError>,
) -> Result<T, EmitError> {
    let saved = cx.if_branch_key;
    cx.if_branch_key = *next_key;
    let result = write(cx);
    *next_key = cx.if_branch_key;
    cx.if_branch_key = saved;
    result
}

pub(super) fn default_branch_key_count(cx: &EmitCx<'_>, children: &Region<'_>) -> u32 {
    let mut walk = cx.walk.clone();
    let skip_ws = children
        .ops
        .iter()
        .any(|op| !matches!(op, Op::Text(_) | Op::Interpolation(_)));
    children.ops.iter().fold(0u32, |count, op| {
        let mut probe = walk.clone();
        let Some(id) = probe.mint() else {
            return count;
        };
        let is_entry = match op {
            Op::If(if_op) => is_slot_if(cx, Some(id), if_op),
            Op::For(for_op) => is_slot_for(cx, Some(id), for_op),
            Op::Element(element) => slot_template_content(element).is_some(),
            _ => false,
        };
        advance_after_op(&mut walk, op);
        if (skip_ws && is_whitespace_text(op)) || is_entry {
            count
        } else {
            let mut nested = walk.clone();
            count.saturating_add(op_branch_key_count(cx, op, &mut nested))
        }
    })
}

fn region_branch_key_count(cx: &EmitCx<'_>, region: &Region<'_>, walk: &mut PageWalk) -> u32 {
    region.ops.iter().fold(0u32, |count, op| {
        count.saturating_add(op_branch_key_count(cx, op, walk))
    })
}

fn op_branch_key_count(cx: &EmitCx<'_>, op: &Op<'_>, walk: &mut PageWalk) -> u32 {
    let id = walk.mint();
    match op {
        Op::Element(element) => {
            walk.skip(element.bindings.len());
            region_branch_key_count(cx, &element.children, walk)
        }
        Op::Component(component) => {
            walk.skip(component.bindings.len());
            region_branch_key_count(cx, &component.children, walk)
        }
        Op::If(if_op) if is_slot_if(cx, id, if_op) => slot_if_branch_key_count(cx, if_op, walk),
        Op::If(if_op) => {
            for branch in if_op.branches.iter() {
                skip_ops_for_count(walk, &branch.region.ops);
            }
            u32::try_from(if_op.branches.len()).unwrap_or(u32::MAX)
        }
        Op::For(for_op) => region_branch_key_count(cx, &for_op.region, walk),
        Op::Slot(slot) => {
            walk.skip(slot.bindings.len());
            region_branch_key_count(cx, &slot.fallback, walk)
        }
        Op::Text(_) | Op::Interpolation(_) => 0,
    }
}

fn slot_if_branch_key_count(cx: &EmitCx<'_>, if_op: &IfOp<'_>, walk: &mut PageWalk) -> u32 {
    if_op.branches.iter().fold(0u32, |count, branch| {
        let Some((idx, element, _content)) = first_slot_template(&branch.region) else {
            skip_ops_for_count(walk, &branch.region.ops);
            return count;
        };
        skip_ops_for_count(walk, &branch.region.ops[..idx]);
        let _id = walk.mint();
        walk.skip(element.bindings.len());
        let count = count.saturating_add(region_branch_key_count(cx, &element.children, walk));
        skip_ops_for_count(walk, &branch.region.ops[idx + 1..]);
        count
    })
}

fn skip_ops_for_count(walk: &mut PageWalk, ops: &[Op<'_>]) {
    for op in ops {
        let _id = walk.mint();
        advance_after_op(walk, op);
    }
}
