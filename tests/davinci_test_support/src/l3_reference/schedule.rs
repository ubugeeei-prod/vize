//! P3-15 scheduling contract. L3 has no separate effect-grouping pass, so
//! every grouping must meet the TS-27 phase validator's edge contract. This
//! restates its order checks without `vize_l3`: L3V001 duplicate op ids, L3V004
//! unresolved endpoints, L3V007 effect-scoped edges leaving their scope and
//! L3V008 scheduled edges that point backward. Each check carries the
//! declarative statement of its code as a contract, and `creusot.rs` proves
//! that acceptance orders every edge in the trace semantics of both backends.

#[cfg(creusot)]
use creusot_std::prelude::vec;
#[cfg(creusot)]
use creusot_std::prelude::*;

use super::program::{Phase, Program, Row, StateEdge, index_of, lookup};
#[cfg(creusot)]
use super::program::{absent, distinct, distinct_before, first_at, found};
use super::trace::TraceEvent;

/// The validator's codes, in the order it reports them.
pub const L3V001: &str = "L3V001";
pub const L3V002: &str = "L3V002";
pub const L3V005: &str = "L3V005";
pub const L3V006: &str = "L3V006";
pub const L3V003: &str = "L3V003";
pub const L3V007: &str = "L3V007";
pub const L3V004: &str = "L3V004";
pub const L3V008: &str = "L3V008";

/// The position of the first op with `id`.
#[cfg_attr(creusot, ensures(match result {
    Some(i) => first_at(program.ops@, id, i@)
        && forall<j: Int> first_at(program.ops@, id, j) ==> j == i@,
    None => absent(program.ops@, id) && forall<j: Int> !first_at(program.ops@, id, j),
}))]
#[must_use]
pub fn op_index(program: &Program, id: u32) -> Option<usize> {
    index_of(&program.ops, id)
}

/// The number of rows whose id already occurred in `seen` or earlier: the
/// validator reports one L3V001 for each. None means the ids are distinct
/// and fresh (Lean `nodup_of_duplicateCodes`).
#[cfg_attr(creusot, ensures((result@ == 0) == (distinct(rows@)
    && forall<i: Int> 0 <= i && i < rows@.len() ==> absent(seen@, rows@[i].key_logic()))))]
#[must_use]
pub fn duplicates<T: Row>(seen: &[u32], rows: &[T]) -> usize {
    let mut count = 0;
    #[cfg_attr(creusot, invariant(count@ <= produced.len()))]
    #[cfg_attr(creusot, invariant((count@ == 0) == (distinct_before(rows@, produced.len())
        && forall<i: Int> 0 <= i && i < produced.len() ==> absent(seen@, rows@[i].key_logic()))))]
    for i in 0..rows.len() {
        // A row repeats an id when `seen` has it or its first row is earlier.
        if let Some(row) = rows.get(i)
            && (index_of(seen, row.key()).is_some() || index_of(rows, row.key()) != Some(i))
        {
            count += 1;
        }
    }
    count
}

/// The region of the first op with `id`.
#[cfg_attr(creusot, ensures(match result {
    Some(region) => found(program.ops@, id)
        && forall<i: Int> first_at(program.ops@, id, i) ==> program.ops@[i].region == region,
    None => absent(program.ops@, id) && !found(program.ops@, id),
}))]
#[must_use]
pub fn op_region(program: &Program, id: u32) -> Option<u32> {
    lookup(&program.ops, id).map(|op| op.region)
}

/// The parent of the first region with `id`, which may itself be absent.
#[cfg_attr(creusot, ensures(match result {
    Some(parent) => found(program.regions@, id)
        && forall<i: Int> first_at(program.regions@, id, i) ==> program.regions@[i].parent == parent,
    None => absent(program.regions@, id) && !found(program.regions@, id),
}))]
#[must_use]
pub fn region_parent(program: &Program, id: u32) -> Option<Option<u32>> {
    lookup(&program.regions, id).map(|region| region.parent)
}

#[cfg_attr(creusot, ensures(match result {
    Some(next) => found(program.regions@, id)
        && forall<i: Int> first_at(program.regions@, id, i) ==> program.regions@[i].parent == Some(next),
    None => forall<i: Int> first_at(program.regions@, id, i) ==> program.regions@[i].parent == None,
}))]
#[must_use]
pub fn parent(program: &Program, id: u32) -> Option<u32> {
    region_parent(program, id).flatten()
}

/// The region of the first effect with `id`.
#[cfg_attr(creusot, ensures(match result {
    Some(region) => found(program.effects@, id)
        && forall<i: Int> first_at(program.effects@, id, i) ==> program.effects@[i].region == region,
    None => absent(program.effects@, id) && !found(program.effects@, id),
}))]
#[must_use]
pub fn effect_region(program: &Program, id: u32) -> Option<u32> {
    lookup(&program.effects, id).map(|effect| effect.region)
}

/// `child` is `ancestor`, or its chain of first-match parents reaches
/// `ancestor`, visiting at most `fuel` regions.
#[cfg(creusot)]
#[logic(open)]
#[variant(fuel)]
pub fn descends(program: Program, fuel: Int, child: u32, ancestor: u32) -> bool {
    pearlite! {
        fuel > 0 && (child == ancestor
            || exists<i: Int> first_at(program.regions@, child, i)
                && match program.regions@[i].parent {
                    Some(next) => descends(program, fuel - 1, next, ancestor),
                    None => false,
                })
    }
}

/// The first op with `id` lives in `region` or below it.
#[cfg(creusot)]
#[logic(open)]
pub fn inside(program: Program, id: u32, region: u32) -> bool {
    pearlite! {
        exists<i: Int> first_at(program.ops@, id, i)
            && descends(program, program.regions@.len() + 1, program.ops@[i].region, region)
    }
}

/// Walk parents, visiting at most one more region than the program has.
#[cfg_attr(creusot, ensures(result
    == descends(*program, program.regions@.len() + 1, child, ancestor)))]
#[must_use]
pub fn is_or_descendant(program: &Program, child: u32, ancestor: u32) -> bool {
    let mut current = child;
    #[cfg_attr(creusot, invariant(descends(*program, program.regions@.len() + 1, child, ancestor)
        == descends(*program, program.regions@.len() + 1 - produced.len(), current, ancestor)))]
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

#[cfg_attr(creusot, ensures(result == inside(*program, id, region)))]
#[must_use]
pub fn op_inside(program: &Program, id: u32, region: u32) -> bool {
    op_region(program, id).is_some_and(|child| is_or_descendant(program, child, region))
}

/// L3V007 for one edge: it names a missing effect or, once both endpoints
/// resolve, an endpoint lies outside the first such effect's region.
#[cfg(creusot)]
#[logic(open)]
pub fn leaves_scope(program: Program, edge: StateEdge, resolved: bool) -> bool {
    pearlite! {
        match edge.effect {
            None => false,
            Some(scope) => !found(program.effects@, scope)
                || resolved && exists<k: Int> first_at(program.effects@, scope, k)
                    && !(inside(program, edge.source, program.effects@[k].region)
                        && inside(program, edge.target, program.effects@[k].region)),
        }
    }
}

/// L3V008 for one edge: in the scheduled phase, the first source op is not
/// before the first target op.
#[cfg(creusot)]
#[logic(open)]
pub fn backward(program: Program, edge: StateEdge) -> bool {
    pearlite! {
        program.phase == Phase::Scheduled
            && exists<i: Int, j: Int> first_at(program.ops@, edge.source, i)
                && first_at(program.ops@, edge.target, j) && i >= j
    }
}

/// The validator reports no code for `edge`.
#[cfg(creusot)]
#[logic(open)]
pub fn clean(program: Program, edge: StateEdge) -> bool {
    pearlite! {
        found(program.ops@, edge.source) && found(program.ops@, edge.target)
            && !leaves_scope(program, edge, true) && !backward(program, edge)
    }
}

/// The validator accepts the ordering: distinct op ids and no code on any
/// edge (Lean `orderCodes_nil`).
#[cfg(creusot)]
#[logic(open)]
pub fn accepted(program: Program) -> bool {
    pearlite! {
        distinct(program.ops@)
            && forall<k: Int> 0 <= k && k < program.edges@.len() ==> clean(program, program.edges@[k])
    }
}

/// L3V007 when an effect-scoped edge names a missing effect or, once both
/// endpoints resolve, leaves the effect's region.
#[cfg_attr(creusot, ensures(result
    == if leaves_scope(*program, *edge, resolved) { Some(L3V007) } else { None }))]
#[must_use]
pub fn scope_code(program: &Program, edge: &StateEdge, resolved: bool) -> Option<&'static str> {
    let scope = edge.effect?;
    match effect_region(program, scope) {
        None => Some(L3V007),
        Some(region) => (resolved
            && !(op_inside(program, edge.source, region)
                && op_inside(program, edge.target, region)))
        .then_some(L3V007),
    }
}

#[cfg_attr(creusot, ensures(result == match (phase, source, target) {
    (Phase::Scheduled, Some(i), Some(j)) => if i@ >= j@ { Some(L3V008) } else { None },
    _ => None,
}))]
#[must_use]
pub fn order_code(
    phase: Phase,
    source: Option<usize>,
    target: Option<usize>,
) -> Option<&'static str> {
    match (phase, source, target) {
        (Phase::Scheduled, Some(i), Some(j)) if i >= j => Some(L3V008),
        _ => None,
    }
}

/// The codes of one edge in validator order: an L3V004 per unresolved
/// endpoint, then L3V007, then L3V008. A fixed array keeps it off the heap.
pub type EdgeCodes = [Option<&'static str>; 4];

#[cfg_attr(creusot, ensures(result@[0]
    == if found(program.ops@, edge.source) { None } else { Some(L3V004) }))]
#[cfg_attr(creusot, ensures(result@[1]
    == if found(program.ops@, edge.target) { None } else { Some(L3V004) }))]
#[cfg_attr(creusot, ensures(result@[2] == if leaves_scope(*program, *edge,
    found(program.ops@, edge.source) && found(program.ops@, edge.target)) {
    Some(L3V007)
} else {
    None
}))]
#[cfg_attr(creusot, ensures(result@[3]
    == if backward(*program, *edge) { Some(L3V008) } else { None }))]
#[must_use]
pub fn edge_codes(program: &Program, edge: &StateEdge) -> EdgeCodes {
    let source = op_index(program, edge.source);
    let target = op_index(program, edge.target);
    let resolved = source.is_some() && target.is_some();
    [
        source.is_none().then_some(L3V004),
        target.is_none().then_some(L3V004),
        scope_code(program, edge, resolved),
        order_code(program.phase, source, target),
    ]
}

/// The validator's ordering verdict: no code means acceptance.
#[cfg_attr(creusot, ensures((result@.len() == 0) == accepted(*program)))]
#[must_use]
pub fn order_codes(program: &Program) -> Vec<&'static str> {
    let mut codes = vec![L3V001; duplicates(&[], &program.ops)];
    #[cfg(creusot)]
    let duplicated = snapshot!(codes@.len() != 0);
    #[cfg_attr(creusot, invariant((codes@.len() == 0) == (!*duplicated
        && forall<k: Int> 0 <= k && k < produced.len() ==> clean(*program, program.edges@[k]))))]
    for k in 0..program.edges.len() {
        if let Some(edge) = program.edges.get(k) {
            let edge = edge_codes(program, edge);
            codes.extend(edge[0]);
            codes.extend(edge[1]);
            codes.extend(edge[2]);
            codes.extend(edge[3]);
        }
    }
    codes
}

/// No slot holds a code.
#[cfg_attr(creusot, ensures(result
    == (codes@[0] == None && codes@[1] == None && codes@[2] == None && codes@[3] == None)))]
#[must_use]
pub fn no_code(codes: &EdgeCodes) -> bool {
    codes[0].is_none() && codes[1].is_none() && codes[2].is_none() && codes[3].is_none()
}

/// Whether `order_codes` is empty, scanned piece by piece: both are
/// `accepted` (`acceptance_is_an_empty_verdict`).
#[cfg_attr(creusot, ensures(result == accepted(*program)))]
#[must_use]
pub fn accepts(program: &Program) -> bool {
    if duplicates(&[], &program.ops) != 0 {
        return false;
    }
    #[cfg_attr(creusot, invariant(forall<k: Int> 0 <= k && k < produced.len()
        ==> clean(*program, program.edges@[k])))]
    for k in 0..program.edges.len() {
        if let Some(edge) = program.edges.get(k)
            && !no_code(&edge_codes(program, edge))
        {
            return false;
        }
    }
    true
}

/// Executable happens-before (Lean `Precedes`): both ops execute, and every
/// event of `source` precedes every event of `target`.
#[cfg(creusot)]
#[logic(open)]
pub fn happens_before(trace: Seq<TraceEvent>, source: u32, target: u32) -> bool {
    pearlite! {
        (exists<a: Int> 0 <= a && a < trace.len() && trace[a].op_id == source)
            && (exists<b: Int> 0 <= b && b < trace.len() && trace[b].op_id == target)
            && forall<a: Int, b: Int> 0 <= a && a < trace.len() && 0 <= b && b < trace.len()
                && trace[a].op_id == source && trace[b].op_id == target ==> a < b
    }
}

/// Executable happens-before: both ops execute, and every event of `source`
/// precedes every event of `target`, so the last source event precedes the
/// first target event.
#[cfg_attr(creusot, ensures(result == happens_before(trace@, source, target)))]
#[must_use]
pub fn precedes(trace: &[TraceEvent], source: u32, target: u32) -> bool {
    let mut last_source: Option<usize> = None;
    let mut first_target: Option<usize> = None;
    #[cfg_attr(creusot, invariant(match last_source {
        None => forall<a: Int> 0 <= a && a < produced.len() ==> trace@[a].op_id != source,
        Some(a) => a@ < produced.len() && trace@[a@].op_id == source
            && forall<b: Int> a@ < b && b < produced.len() ==> trace@[b].op_id != source,
    }))]
    #[cfg_attr(creusot, invariant(match first_target {
        None => forall<b: Int> 0 <= b && b < produced.len() ==> trace@[b].op_id != target,
        Some(b) => b@ < produced.len() && trace@[b@].op_id == target
            && forall<a: Int> 0 <= a && a < b@ ==> trace@[a].op_id != target,
    }))]
    for i in 0..trace.len() {
        if let Some(event) = trace.get(i) {
            if event.op_id == source {
                last_source = Some(i);
            }
            if event.op_id == target && first_target.is_none() {
                first_target = Some(i);
            }
        }
    }
    matches!((last_source, first_target), (Some(i), Some(j)) if i < j)
}
