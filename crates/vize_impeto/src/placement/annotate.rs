//! The canonical placement enumerator.
//!
//! It records every alternative the verifier accepts, except interior hoist
//! roots: a hoist is offered only at the outermost static element of a
//! controlled subtree, because the outer choice already covers every op the
//! inner one could move. Apart from the bounded parent and predecessor walks
//! the verifier also uses, each op is visited a constant number of times.

use alloc::vec::Vec;

use super::facts::{Index, contains};
use super::{Placement, PlacementRecord, PlacementSet};
use crate::op::{OpId, OpKind, Program, RegionId};

/// Replace `program.placements` with the canonical alternatives of every op,
/// each still choosing [`Placement::Inline`].
///
/// This never touches ops, regions, edges, effects, or operands, so partition
/// facts exported for the canonical program stay exact.
pub fn annotate(program: &mut Program<'_>) {
    let records = enumerate(&Index::new(program));
    program.placements.clear();
    program.placements.extend(records);
}

struct Shapes {
    statics: Vec<bool>,
    /// Regions holding a non-static op directly or in a nested region.
    open: Vec<bool>,
    /// `(region, position)` of every non-static op.
    dynamic: Vec<(RegionId, u32)>,
}

fn enumerate(index: &Index<'_, '_>) -> Vec<PlacementRecord> {
    let program = index.program;
    let shapes = shapes(index);
    let hoistable: Vec<bool> = (0..program.ops.len())
        .map(|position| hoistable(index, &shapes, position))
        .collect();
    let mut leaders: Vec<Option<OpId>> = Vec::with_capacity(program.ops.len());
    let mut records = Vec::new();
    for (position, op) in program.ops.iter().enumerate() {
        let leader = group_leader(index, &leaders, position);
        leaders.push(leader);
        let alternative = if hoistable[position] && !inside_hoistable(index, &hoistable, position) {
            Some(Placement::Hoist)
        } else if op.kind == OpKind::SetEvent
            && op.effect.is_some()
            && index.plain_handler(op)
            && index.scope_free(op.region)
        {
            Some(Placement::Cache)
        } else if leader.is_some() {
            Some(Placement::Group)
        } else {
            None
        };
        if let Some(alternative) = alternative {
            let set = PlacementSet::INLINE.with(alternative);
            records.push(PlacementRecord::new(op.id, set, leader));
        }
    }
    records
}

fn shapes(index: &Index<'_, '_>) -> Shapes {
    let program = index.program;
    let statics: Vec<bool> = program
        .ops
        .iter()
        .map(|op| index.shape_static(op))
        .collect();
    let mut open = alloc::vec![false; program.regions.len()];
    let mut dynamic = Vec::new();
    for (position, op) in program.ops.iter().enumerate() {
        if statics[position] {
            continue;
        }
        dynamic.push((op.region, position as u32));
        let mut current = Some(op.region);
        for _ in 0..=program.regions.len() {
            let Some(region) = current.and_then(|id| index.region_position(id)) else {
                break;
            };
            if open[region] {
                break;
            }
            open[region] = true;
            current = program.regions[region].parent;
        }
    }
    dynamic.sort_by_key(|entry| entry.0);
    Shapes {
        statics,
        open,
        dynamic,
    }
}

fn hoistable(index: &Index<'_, '_>, shapes: &Shapes, position: usize) -> bool {
    let op = &index.program.ops[position];
    if op.kind != OpKind::InsertNode
        || !shapes.statics[position]
        || index.position(op.id) != Some(position)
        || index.controller(op.region).is_none()
    {
        return false;
    }
    let children_static = index.owned_regions(op.id).all(|region| {
        index
            .region_position(region)
            .is_some_and(|region| !shapes.open[region])
    });
    let start = shapes.dynamic.partition_point(|entry| entry.0 < op.region);
    let attached_static = shapes.dynamic[start..]
        .iter()
        .take_while(|entry| entry.0 == op.region)
        .all(|entry| !contains(op.span, index.program.ops[entry.1 as usize].span));
    children_static && attached_static
}

/// Whether the element owning this op's region is itself a hoist root.
fn inside_hoistable(index: &Index<'_, '_>, hoistable: &[bool], position: usize) -> bool {
    index
        .region(index.program.ops[position].region)
        .and_then(|region| region.owner)
        .and_then(|owner| index.position(owner))
        .is_some_and(|owner| hoistable[owner])
}

fn group_leader(index: &Index<'_, '_>, leaders: &[Option<OpId>], position: usize) -> Option<OpId> {
    let ops = &index.program.ops;
    let op = &ops[position];
    let reference = index.reference(op)?;
    let pred = index.keyed_pred(op.id)?;
    let pred_position = index.position(pred)?;
    if pred_position >= position || index.reference(&ops[pred_position])? != reference {
        return None;
    }
    let scope = index.group_scope(op.region)?;
    if index.group_scope(ops[pred_position].region)? != scope {
        return None;
    }
    Some(leaders[pred_position].unwrap_or(pred))
}
