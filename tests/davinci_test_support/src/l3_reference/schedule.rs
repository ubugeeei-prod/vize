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
pub fn index_of(ids: &[u32], id: u32) -> Option<usize> {
    ids.iter().position(|other| *other == id)
}

/// One L3V001 for every id that already occurred in `seen` or earlier.
#[must_use]
pub fn duplicate_codes(seen: &[u32], ids: &[u32]) -> Vec<&'static str> {
    let mut seen = seen.to_vec();
    let mut codes = Vec::new();
    for id in ids {
        if seen.contains(id) {
            codes.push("L3V001");
        }
        seen.push(*id);
    }
    codes
}

#[must_use]
pub fn op_region(program: &Program, id: u32) -> Option<u32> {
    program.op(id).map(|op| op.region)
}

#[must_use]
pub fn parent(program: &Program, id: u32) -> Option<u32> {
    program.region(id).and_then(|region| region.parent)
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

#[must_use]
pub fn scope_codes(program: &Program, edge: &StateEdge, resolved: bool) -> Vec<&'static str> {
    let Some(scope) = edge.effect else {
        return Vec::new();
    };
    match program.effect(scope) {
        None => vec!["L3V007"],
        Some(effect)
            if resolved
                && !(op_inside(program, edge.source, effect.region)
                    && op_inside(program, edge.target, effect.region)) =>
        {
            vec!["L3V007"]
        }
        Some(_) => Vec::new(),
    }
}

#[must_use]
pub fn order_code(phase: Phase, source: Option<usize>, target: Option<usize>) -> Vec<&'static str> {
    match (phase, source, target) {
        (Phase::Scheduled, Some(i), Some(j)) if i >= j => vec!["L3V008"],
        _ => Vec::new(),
    }
}

#[must_use]
pub fn edge_codes(program: &Program, edge: &StateEdge) -> Vec<&'static str> {
    let ids = program.ids();
    let source = index_of(&ids, edge.source);
    let target = index_of(&ids, edge.target);
    let mut codes = Vec::new();
    if source.is_none() {
        codes.push("L3V004");
    }
    if target.is_none() {
        codes.push("L3V004");
    }
    codes.extend(scope_codes(
        program,
        edge,
        source.is_some() && target.is_some(),
    ));
    codes.extend(order_code(program.phase, source, target));
    codes
}

/// The validator's ordering verdict: no code means acceptance.
#[must_use]
pub fn order_codes(program: &Program) -> Vec<&'static str> {
    let mut codes = duplicate_codes(&[], &program.ids());
    for edge in &program.edges {
        codes.extend(edge_codes(program, edge));
    }
    codes
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
