//! P3-15 scheduling contract. L3 has no separate effect-grouping pass, so
//! every grouping must meet the TS-27 phase validator's edge contract. This
//! restates its order checks without `vize_l3`: L3V001 duplicate op ids, L3V004
//! unresolved endpoints, L3V007 effect-scoped edges leaving their scope and
//! L3V008 scheduled edges that point backward. `kani.rs` proves acceptance
//! orders every edge in the trace semantics of both backends.

use super::program::{Phase, Program, StateEdge};
use super::trace::TraceEvent;

/// First position of `id`, as the validator's `op_index` computes it.
#[must_use]
pub fn index_of(ids: impl IntoIterator<Item = u32>, id: u32) -> Option<usize> {
    ids.into_iter().position(|other| other == id)
}

/// The position of the first op with `id`.
#[must_use]
pub fn op_index(program: &Program, id: u32) -> Option<usize> {
    index_of(program.ops.iter().map(|op| op.id), id)
}

/// One L3V001 for every item whose id already occurred in `seen` or earlier.
pub fn duplicate_codes<'i, T>(
    seen: &'i [u32],
    items: &'i [T],
    id: impl Fn(&T) -> u32 + Copy + 'i,
) -> impl Iterator<Item = &'static str> + 'i {
    (items.iter().enumerate())
        .filter(move |(index, item)| {
            let earlier = items.get(..*index).unwrap_or_default();
            // Element scans, not `contains`: the integer fast path costs Kani 5x.
            seen.iter().any(|other| *other == id(item))
                || earlier.iter().any(|other| id(other) == id(item))
        })
        .map(|_| "L3V001")
}

// Lookups return values, not references: CBMC encodes a symbolic reference
// into a table far less cheaply than the value read in the scan.

/// The region of the first op with `id`.
#[must_use]
pub fn op_region(program: &Program, id: u32) -> Option<u32> {
    (program.ops.iter()).find_map(|op| (op.id == id).then_some(op.region))
}

/// The parent of the first region with `id`, which may itself be absent.
#[must_use]
pub fn region_parent(program: &Program, id: u32) -> Option<Option<u32>> {
    (program.regions.iter()).find_map(|region| (region.id == id).then_some(region.parent))
}

#[must_use]
pub fn parent(program: &Program, id: u32) -> Option<u32> {
    region_parent(program, id).flatten()
}

/// The region of the first effect with `id`.
#[must_use]
pub fn effect_region(program: &Program, id: u32) -> Option<u32> {
    (program.effects.iter()).find_map(|effect| (effect.id == id).then_some(effect.region))
}

/// Walk parents, visiting at most one more region than the program has.
#[must_use]
pub fn is_or_descendant(program: &Program, child: u32, ancestor: u32) -> bool {
    let mut current = child;
    for _ in 0..=program.regions.len() {
        if current == ancestor {
            return true;
        }
        match parent(program, current) {
            Some(next) => current = next,
            None => return false,
        }
    }
    false
}

#[must_use]
pub fn op_inside(program: &Program, id: u32, region: u32) -> bool {
    op_region(program, id).is_some_and(|child| is_or_descendant(program, child, region))
}

/// L3V007 when an effect-scoped edge names a missing effect or, once both
/// endpoints resolve, leaves the effect's region.
#[must_use]
pub fn scope_code(program: &Program, edge: &StateEdge, resolved: bool) -> Option<&'static str> {
    let scope = edge.effect?;
    match effect_region(program, scope) {
        None => Some("L3V007"),
        Some(region) => (resolved
            && !(op_inside(program, edge.source, region)
                && op_inside(program, edge.target, region)))
        .then_some("L3V007"),
    }
}

#[must_use]
pub fn order_code(
    phase: Phase,
    source: Option<usize>,
    target: Option<usize>,
) -> Option<&'static str> {
    match (phase, source, target) {
        (Phase::Scheduled, Some(i), Some(j)) if i >= j => Some("L3V008"),
        _ => None,
    }
}

/// The codes of one edge in validator order: an L3V004 per unresolved
/// endpoint, then L3V007, then L3V008. A fixed array keeps it off the heap.
pub type EdgeCodes = [Option<&'static str>; 4];

#[must_use]
pub fn edge_codes(program: &Program, edge: &StateEdge) -> EdgeCodes {
    let source = op_index(program, edge.source);
    let target = op_index(program, edge.target);
    let resolved = source.is_some() && target.is_some();
    [
        source.is_none().then_some("L3V004"),
        target.is_none().then_some("L3V004"),
        scope_code(program, edge, resolved),
        order_code(program.phase, source, target),
    ]
}

/// The validator's ordering verdict: no code means acceptance.
#[must_use]
pub fn order_codes(program: &Program) -> Vec<&'static str> {
    let mut codes: Vec<&'static str> = duplicate_codes(&[], &program.ops, |op| op.id).collect();
    for edge in &program.edges {
        let [source, target, scope, order] = edge_codes(program, edge);
        codes.extend(source);
        codes.extend(target);
        codes.extend(scope);
        codes.extend(order);
    }
    codes
}

/// Whether `order_codes` is empty, scanned piece by piece: Kani proves the
/// laws over this form and `acceptance_is_an_empty_verdict` ties the two.
#[must_use]
pub fn accepts(program: &Program) -> bool {
    duplicate_codes(&[], &program.ops, |op| op.id)
        .next()
        .is_none()
        && (program.edges.iter())
            .all(|edge| matches!(edge_codes(program, edge), [None, None, None, None]))
}

/// Executable happens-before: both ops execute, and every event of `source`
/// precedes every event of `target`.
#[must_use]
pub fn precedes(trace: &[TraceEvent], source: u32, target: u32) -> bool {
    let at = |id: u32| {
        trace
            .iter()
            .enumerate()
            .filter(move |(_, event)| event.op_id == id)
            .map(|(index, _)| index)
    };
    at(source).next().is_some()
        && at(target).next().is_some()
        && at(source).all(|i| at(target).all(|j| i < j))
}
