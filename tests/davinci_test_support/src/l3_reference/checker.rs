//! C-23 independent L3 graph checker. It re-checks the graph invariants the
//! TS-27 phase validator enforces from the committed page alone and shares no
//! code with the compiler. Every check is total. `kani.rs` proves the checker
//! exact against a declarative statement of each invariant.

use super::program::{Phase, Program, Span, StateEdge};
use super::schedule::{is_or_descendant, op_index, op_inside, region_parent};

#[must_use]
pub fn contains(outer: Span, inner: Span) -> bool {
    outer.start <= inner.start && inner.stop <= outer.stop
}

/// The parent chain from `region` reaches a parentless region within `fuel` steps.
#[must_use]
pub fn reaches_top(program: &Program, fuel: usize, region: u32) -> bool {
    let mut current = region;
    for _ in 0..fuel {
        match region_parent(program, current) {
            Some(Some(parent)) => current = parent,
            Some(None) => return true,
            None => return false,
        }
    }
    false
}

#[must_use]
pub fn forward(program: &Program, edge: &StateEdge) -> bool {
    match (
        op_index(program, edge.source),
        op_index(program, edge.target),
    ) {
        (Some(i), Some(j)) => i < j,
        _ => false,
    }
}

/// No two items share an id.
fn unique<T>(items: &[T], id: impl Fn(&T) -> u32) -> bool {
    (items.iter().enumerate()).all(|(index, item)| {
        !items
            .iter()
            .skip(index + 1)
            .any(|other| id(other) == id(item))
    })
}

/// L3V001: op, region and effect ids are unique.
#[must_use]
pub fn unique_ids(p: &Program) -> bool {
    unique(&p.ops, |o| o.id) && unique(&p.regions, |r| r.id) && unique(&p.effects, |e| e.id)
}

/// L3V002: region 0 exists and is the parentless, ownerless root.
#[must_use]
pub fn rooted(p: &Program) -> bool {
    p.regions
        .iter()
        .any(|r| r.id == 0 && r.parent.is_none() && r.owner.is_none())
}

/// L3V005: every other region names a resolvable parent and an owner op in it.
#[must_use]
pub fn resolved(p: &Program) -> bool {
    p.regions.iter().filter(|r| r.id != 0).all(|r| {
        p.regions.iter().any(|q| r.parent == Some(q.id))
            && p.ops
                .iter()
                .any(|o| r.owner == Some(o.id) && r.parent == Some(o.region))
    })
}

/// L3V006: spans nest (region in parent, region in owner, op in region) and
/// every parent chain is finite.
#[must_use]
pub fn nested(p: &Program) -> bool {
    let fuel = p.regions.len() + 1;
    p.regions.iter().all(|r| {
        p.regions
            .iter()
            .all(|q| r.parent != Some(q.id) || contains(q.span, r.span))
            && p.ops
                .iter()
                .all(|o| r.owner != Some(o.id) || contains(o.span, r.span))
            && reaches_top(p, fuel, r.id)
    }) && p.ops.iter().all(|o| {
        p.regions
            .iter()
            .all(|r| r.id != o.region || contains(r.span, o.span))
    })
}

/// L3V003: ops live in existing regions.
#[must_use]
pub fn ops_resolved(p: &Program) -> bool {
    p.ops
        .iter()
        .all(|o| p.regions.iter().any(|r| r.id == o.region))
}

/// L3V007: ops name existing effects; an effect's owner lives in its region
/// (or below) and its span nests in that region.
#[must_use]
pub fn scoped(p: &Program) -> bool {
    p.ops
        .iter()
        .all(|o| o.effect.is_none_or(|e| p.effects.iter().any(|s| s.id == e)))
        && p.effects.iter().all(|s| {
            p.ops
                .iter()
                .any(|o| o.id == s.owner && is_or_descendant(p, o.region, s.region))
                && p.regions
                    .iter()
                    .any(|r| r.id == s.region && contains(r.span, s.span))
        })
}

/// L3V004 / L3V007: edges resolve and effect-scoped edges stay in scope.
#[must_use]
pub fn edges_resolved(p: &Program) -> bool {
    let resolves = |id: u32| p.ops.iter().any(|o| o.id == id);
    p.edges.iter().all(|e| {
        resolves(e.source)
            && resolves(e.target)
            && e.effect.is_none_or(|scope| {
                p.effects.iter().any(|s| {
                    s.id == scope
                        && op_inside(p, e.source, s.region)
                        && op_inside(p, e.target, s.region)
                })
            })
    })
}

/// L3V008: in the scheduled phase every edge points strictly forward.
#[must_use]
pub fn ordered(p: &Program) -> bool {
    p.phase != Phase::Scheduled || p.edges.iter().all(|e| forward(p, e))
}

/// Each invariant with its code, in validator order.
#[must_use]
pub fn checks(p: &Program) -> [(bool, &'static str); 8] {
    [
        (unique_ids(p), "L3V001"),
        (rooted(p), "L3V002"),
        (resolved(p), "L3V005"),
        (nested(p), "L3V006"),
        (ops_resolved(p), "L3V003"),
        (scoped(p), "L3V007"),
        (edges_resolved(p), "L3V004"),
        (ordered(p), "L3V008"),
    ]
}

/// One code per violated invariant, in validator order.
#[must_use]
pub fn violations(p: &Program) -> Vec<&'static str> {
    let checks = checks(p);
    checks
        .iter()
        .filter(|(holds, _)| !holds)
        .map(|(_, code)| *code)
        .collect()
}
