use alloc::vec::Vec as StdVec;

use vize_s0::String;
use vize_s2::op::{Op, Region};

use super::{capture_child, emit_template_pieces, is_slot_template};
use crate::emit::{EmitCx, EmitError, UnsupportedReason as Reason};
use crate::pass::walk::PageWalk;
use crate::pass::{SlotCarrier, SlotFacts, SlotParams};

pub(super) fn collect_pieces(
    cx: &mut EmitCx<'_>,
    children: &Region<'_>,
    facts: &SlotFacts,
    buckets: &mut [StdVec<String>],
) -> Result<(), EmitError> {
    let mut group_keys = group_branch_key_starts(cx, children, facts);
    if children
        .ops
        .iter()
        .all(|op| matches!(op, Op::Text(_) | Op::Interpolation(_)))
        && let Some(idx) = facts
            .groups
            .iter()
            .position(|group| matches!(group.carrier, SlotCarrier::Component))
    {
        let scoped = matches!(facts.groups[idx].params, SlotParams::Scoped { .. });
        with_group_if_key(cx, &mut group_keys, idx, |cx| {
            crate::emit::outlet::with_slot_params(cx, scoped, |cx| {
                emit_template_pieces(cx, children, &mut buckets[idx])
            })
        })?;
        if let Some(after) = group_keys.last().copied() {
            cx.if_branch_key = after;
        }
        return Ok(());
    }
    for op in children.ops.iter() {
        match op {
            Op::Element(element) if is_slot_template(element) => {
                let id = cx.walk.mint();
                cx.walk.skip(element.bindings.len());
                let Some(idx) = facts.groups.iter().position(
                    |group| matches!(group.carrier, SlotCarrier::Template(tid) if tid == id),
                ) else {
                    return Err(EmitError::unsupported_at(
                        Reason::SlotFactsMissingGroup,
                        element.span,
                    ));
                };
                let scoped = matches!(facts.groups[idx].params, SlotParams::Scoped { .. });
                with_group_if_key(cx, &mut group_keys, idx, |cx| {
                    crate::emit::outlet::with_slot_params(cx, scoped, |cx| {
                        emit_template_pieces(cx, &element.children, &mut buckets[idx])
                    })
                })?;
            }
            _ => {
                let idx = facts.groups.iter().position(|group| {
                    matches!(
                        group.carrier,
                        SlotCarrier::Implicit | SlotCarrier::Component
                    )
                });
                let Some(idx) = idx else {
                    return Err(EmitError::unsupported_op(Reason::SlotFactsMissingGroup, op));
                };
                let scoped = matches!(facts.groups[idx].params, SlotParams::Scoped { .. });
                let piece = with_group_if_key(cx, &mut group_keys, idx, |cx| {
                    crate::emit::outlet::with_slot_params(cx, scoped, |cx| capture_child(cx, op))
                })?;
                buckets[idx].push(piece);
            }
        }
    }
    if let Some(after) = group_keys.last().copied() {
        cx.if_branch_key = after;
    }
    Ok(())
}

fn group_branch_key_starts(
    cx: &EmitCx<'_>,
    children: &Region<'_>,
    facts: &SlotFacts,
) -> StdVec<u32> {
    let mut counts = facts.groups.iter().map(|_| 0u32).collect::<StdVec<_>>();
    let mut walk = cx.walk.clone();
    for op in children.ops.iter() {
        match op {
            Op::Element(element) if is_slot_template(element) => {
                let id = walk.mint();
                if let Some(idx) = facts.groups.iter().position(
                    |group| matches!(group.carrier, SlotCarrier::Template(tid) if tid == id),
                ) {
                    let mut nested = walk.clone();
                    let _id = nested.mint();
                    nested.skip(element.bindings.len());
                    counts[idx] = counts[idx].saturating_add(region_branch_key_count(
                        cx,
                        &element.children,
                        &mut nested,
                    ));
                }
                crate::emit::create_slots_walk::advance_after_op(&mut walk, op);
            }
            _ => {
                if let Some(idx) = facts.groups.iter().position(|group| {
                    matches!(
                        group.carrier,
                        SlotCarrier::Implicit | SlotCarrier::Component
                    )
                }) {
                    let mut nested = walk.clone();
                    counts[idx] =
                        counts[idx].saturating_add(op_branch_key_count(cx, op, &mut nested));
                }
                let _id = walk.mint();
                crate::emit::create_slots_walk::advance_after_op(&mut walk, op);
            }
        }
    }

    let mut next = cx.if_branch_key;
    counts
        .iter()
        .map(|count| {
            let start = next;
            next = next.saturating_add(*count);
            start
        })
        .collect()
}

fn with_group_if_key<T>(
    cx: &mut EmitCx<'_>,
    group_keys: &mut [u32],
    idx: usize,
    write: impl FnOnce(&mut EmitCx<'_>) -> Result<T, EmitError>,
) -> Result<T, EmitError> {
    let saved = cx.if_branch_key;
    cx.if_branch_key = group_keys[idx];
    let result = write(cx);
    group_keys[idx] = cx.if_branch_key;
    cx.if_branch_key = saved;
    result
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
        Op::If(if_op) if crate::emit::create_slots_walk::is_slot_if(cx, id, if_op) => 0,
        Op::If(if_op) => u32::try_from(if_op.branches.len()).unwrap_or(u32::MAX),
        Op::For(for_op) => region_branch_key_count(cx, &for_op.region, walk),
        Op::Slot(slot) => {
            walk.skip(slot.bindings.len());
            region_branch_key_count(cx, &slot.fallback, walk)
        }
        Op::Text(_) | Op::Interpolation(_) => 0,
    }
}
