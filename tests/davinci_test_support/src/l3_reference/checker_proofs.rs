//! Exactness of the C-23 checker, `checker.rs`: it reports a code exactly
//! for each violated invariant of the declarative `WellFormed`
//! specification, in validator order. Replaces `Impeto.CheckerLaws`. Bound:
//! the graphs of `graphs.rs` (3 ops, 3 regions, 3 edges, 2 effects, ids
//! below 4, unconstrained spans).

use super::checker::{forward, violations};
use super::graphs;
use super::program::{Phase, Program, Span};
use super::schedule::{index_of, is_or_descendant, op_inside};

fn within(outer: Span, inner: Span) -> bool {
    outer.start <= inner.start && inner.stop <= outer.stop
}

fn distinct(ids: &[u32]) -> bool {
    ids.iter()
        .enumerate()
        .all(|(i, a)| ids.iter().skip(i + 1).all(|b| a != b))
}

/// The parent chain reaches a parentless region within `fuel` first-match
/// lookups (Lean `reachesTop`), stated recursively.
fn chain_ends(p: &Program, fuel: usize, region: u32) -> bool {
    if fuel == 0 {
        return false;
    }
    match p.regions.iter().find(|r| r.id == region) {
        Some(found) => match found.parent {
            Some(parent) => chain_ends(p, fuel - 1, parent),
            None => true,
        },
        None => false,
    }
}

fn unique_ids(p: &Program) -> bool {
    let regions: Vec<u32> = p.regions.iter().map(|r| r.id).collect();
    let effects: Vec<u32> = p.effects.iter().map(|e| e.id).collect();
    distinct(&p.ids()) && distinct(&regions) && distinct(&effects)
}

fn rooted(p: &Program) -> bool {
    p.regions
        .iter()
        .any(|r| r.id == 0 && r.parent.is_none() && r.owner.is_none())
}

fn resolved(p: &Program) -> bool {
    p.regions.iter().all(|r| {
        r.id == 0
            || (p.regions.iter().any(|q| r.parent == Some(q.id))
                && p.ops
                    .iter()
                    .any(|o| r.owner == Some(o.id) && r.parent == Some(o.region)))
    })
}

fn nested(p: &Program) -> bool {
    let parents = p.regions.iter().all(|r| {
        p.regions
            .iter()
            .all(|q| r.parent != Some(q.id) || within(q.span, r.span))
    });
    let owners = p.regions.iter().all(|r| {
        p.ops
            .iter()
            .all(|o| r.owner != Some(o.id) || within(o.span, r.span))
    });
    let ops = p.ops.iter().all(|o| {
        p.regions
            .iter()
            .all(|r| r.id != o.region || within(r.span, o.span))
    });
    let finite = p
        .regions
        .iter()
        .all(|r| chain_ends(p, p.regions.len() + 1, r.id));
    parents && owners && ops && finite
}

fn ops_resolved(p: &Program) -> bool {
    p.ops
        .iter()
        .all(|o| p.regions.iter().any(|r| r.id == o.region))
}

fn scoped(p: &Program) -> bool {
    let named = p.ops.iter().all(|o| match o.effect {
        Some(e) => p.effects.iter().any(|s| s.id == e),
        None => true,
    });
    let owned = p.effects.iter().all(|s| {
        p.ops
            .iter()
            .any(|o| o.id == s.owner && is_or_descendant(p, o.region, s.region))
            && p.regions
                .iter()
                .any(|r| r.id == s.region && within(r.span, s.span))
    });
    named && owned
}

fn edges_resolved(p: &Program) -> bool {
    p.edges.iter().all(|e| {
        p.ops.iter().any(|o| o.id == e.source)
            && p.ops.iter().any(|o| o.id == e.target)
            && match e.effect {
                Some(sc) => p.effects.iter().any(|s| {
                    s.id == sc
                        && op_inside(p, e.source, s.region)
                        && op_inside(p, e.target, s.region)
                }),
                None => true,
            }
    })
}

fn points_forward(p: &Program, source: u32, target: u32) -> bool {
    let ids = p.ids();
    match (index_of(&ids, source), index_of(&ids, target)) {
        (Some(i), Some(j)) => i < j,
        _ => false,
    }
}

fn ordered(p: &Program) -> bool {
    p.phase != Phase::Scheduled
        || p.edges
            .iter()
            .all(|e| points_forward(p, e.source, e.target))
}

/// The specification's verdict, invariant by invariant, in validator order.
fn spec(p: &Program) -> [(bool, &'static str); 8] {
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

/// `code_nil`, `violations_nil_iff`, `violation_sound`: the checker reports
/// exactly the codes of the violated invariants, so it reports nothing
/// exactly when the program is well formed.
#[kani::proof]
#[kani::unwind(6)]
fn the_checker_reports_exactly_the_violated_invariants() {
    let p = graphs::program();
    let expected: Vec<&'static str> = spec(&p)
        .iter()
        .filter(|(holds, _)| !holds)
        .map(|(_, code)| *code)
        .collect();
    let actual = violations(&p);
    let well_formed = spec(&p).iter().all(|(holds, _)| *holds);
    assert!(actual == expected);
    assert!(actual.is_empty() == well_formed);
}

/// `wellFormed_ordered`: in the scheduled phase a well-formed program yields
/// the edge-ordering theorem's hypothesis for every edge.
#[kani::proof]
#[kani::unwind(6)]
fn a_well_formed_schedule_points_every_edge_forward() {
    let p = graphs::program();
    if violations(&p).is_empty() && p.phase == Phase::Scheduled {
        for edge in &p.edges {
            assert!(forward(&p, edge));
        }
    }
}
