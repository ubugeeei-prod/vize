//! Exactness of the C-23 checker, `checker.rs`: it reports a code exactly
//! for each violated invariant of the declarative `WellFormed`
//! specification, in validator order. Replaces `Impeto.CheckerLaws`. Bound:
//! the graphs of `graphs.rs` (3 ops, 2 regions, 1 edge, 1 effect, ids
//! below 3, unconstrained spans).

use super::checker::{self, checks, forward};
use super::graphs;
use super::program::{Phase, Program, Span};
use super::schedule::{index_of, is_or_descendant, op_inside};

fn within(outer: Span, inner: Span) -> bool {
    outer.start <= inner.start && inner.stop <= outer.stop
}

/// `List.Nodup` of the ids: no two positions share one.
fn distinct<T>(items: &[T], id: impl Fn(&T) -> u32) -> bool {
    let mut i = 0;
    while i < items.len() {
        let mut j = i + 1;
        while j < items.len() {
            if items.get(i).map(&id) == items.get(j).map(&id) {
                return false;
            }
            j += 1;
        }
        i += 1;
    }
    true
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
    distinct(&p.ops, |o| o.id) && distinct(&p.regions, |r| r.id) && distinct(&p.effects, |e| e.id)
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
    let ids = || p.ops.iter().map(|o| o.id);
    match (index_of(ids(), source), index_of(ids(), target)) {
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

/// The codes in validator order, as `Impeto.Checker.violations` lists them.
const CODES: [&str; 8] = [
    "L3V001", "L3V002", "L3V005", "L3V006", "L3V003", "L3V007", "L3V004", "L3V008",
];

// `code_nil`, `violations_nil_iff`, `violation_sound`: `violations` keeps the
// code of each failed entry of `checks`, in order. So it is exact when every
// entry is the declarative invariant and the codes follow validator order.
// One harness per invariant keeps each formula small.

/// The entries of `checks` carry the codes in validator order. The codes are
/// literals, independent of the program, so the empty program shows them.
#[kani::proof]
#[kani::unwind(9)]
#[kani::solver(cadical)]
fn the_checker_reports_codes_in_validator_order() {
    let p = Program::empty();
    let [
        (_, a),
        (_, b),
        (_, c),
        (_, d),
        (_, e),
        (_, f),
        (_, g),
        (_, h),
    ] = checks(&p);
    assert!([a, b, c, d, e, f, g, h] == CODES);
}

/// L3V001: the checker's `unique_ids` is the declarative invariant.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn l3v001_unique_ids_is_exact() {
    let p = graphs::program();
    assert!(checker::unique_ids(&p) == unique_ids(&p));
}

/// L3V002: the checker's `rooted` is the declarative invariant.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn l3v002_rooted_is_exact() {
    let p = graphs::program();
    assert!(checker::rooted(&p) == rooted(&p));
}

/// L3V005: the checker's `resolved` is the declarative invariant.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn l3v005_resolved_is_exact() {
    let p = graphs::program();
    assert!(checker::resolved(&p) == resolved(&p));
}

/// L3V003: the checker's `ops_resolved` is the declarative invariant.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn l3v003_ops_resolved_is_exact() {
    let p = graphs::program();
    assert!(checker::ops_resolved(&p) == ops_resolved(&p));
}

/// L3V008: the checker's `ordered` is the declarative invariant.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn l3v008_ordered_is_exact() {
    let p = graphs::program();
    assert!(checker::ordered(&p) == ordered(&p));
}

// Invariants that nest a parent walk inside quantifiers are checked once per
// table size up to the bound: with every size concrete, CBMC unrolls each
// scan exactly, where symbolic sizes cost it an order of magnitude.
macro_rules! exact_at_sizes {
    ($invariant:ident, $code:ident: $(($name:ident, $o:literal, $r:literal, $e:literal, $f:literal)),* $(,)?) => {
        $(
            #[kani::proof]
            #[kani::unwind(4)]
            #[kani::solver(cadical)]
            fn $name() {
                let p = graphs::fixed::<$o, $r, $e, $f>();
                assert!(checker::$invariant(&p) == $invariant(&p));
            }
        )*
    };
}

/// L3V006: the checker's `nested` is the declarative invariant, at every size.
exact_at_sizes!(nested, l3v006:
    (l3v006_nested_at_0_0_0_0, 0, 0, 0, 0),
    (l3v006_nested_at_0_1_0_0, 0, 1, 0, 0),
    (l3v006_nested_at_0_2_0_0, 0, 2, 0, 0),
    (l3v006_nested_at_1_0_0_0, 1, 0, 0, 0),
    (l3v006_nested_at_1_1_0_0, 1, 1, 0, 0),
    (l3v006_nested_at_1_2_0_0, 1, 2, 0, 0),
    (l3v006_nested_at_2_0_0_0, 2, 0, 0, 0),
    (l3v006_nested_at_2_1_0_0, 2, 1, 0, 0),
    (l3v006_nested_at_2_2_0_0, 2, 2, 0, 0),
    (l3v006_nested_at_3_0_0_0, 3, 0, 0, 0),
    (l3v006_nested_at_3_1_0_0, 3, 1, 0, 0),
    (l3v006_nested_at_3_2_0_0, 3, 2, 0, 0),
);

/// L3V007: the checker's `scoped` is the declarative invariant, at every size.
exact_at_sizes!(scoped, l3v007:
    (l3v007_scoped_at_0_0_0_0, 0, 0, 0, 0),
    (l3v007_scoped_at_0_0_0_1, 0, 0, 0, 1),
    (l3v007_scoped_at_0_1_0_0, 0, 1, 0, 0),
    (l3v007_scoped_at_0_1_0_1, 0, 1, 0, 1),
    (l3v007_scoped_at_0_2_0_0, 0, 2, 0, 0),
    (l3v007_scoped_at_0_2_0_1, 0, 2, 0, 1),
    (l3v007_scoped_at_1_0_0_0, 1, 0, 0, 0),
    (l3v007_scoped_at_1_0_0_1, 1, 0, 0, 1),
    (l3v007_scoped_at_1_1_0_0, 1, 1, 0, 0),
    (l3v007_scoped_at_1_1_0_1, 1, 1, 0, 1),
    (l3v007_scoped_at_1_2_0_0, 1, 2, 0, 0),
    (l3v007_scoped_at_1_2_0_1, 1, 2, 0, 1),
    (l3v007_scoped_at_2_0_0_0, 2, 0, 0, 0),
    (l3v007_scoped_at_2_0_0_1, 2, 0, 0, 1),
    (l3v007_scoped_at_2_1_0_0, 2, 1, 0, 0),
    (l3v007_scoped_at_2_1_0_1, 2, 1, 0, 1),
    (l3v007_scoped_at_2_2_0_0, 2, 2, 0, 0),
    (l3v007_scoped_at_2_2_0_1, 2, 2, 0, 1),
    (l3v007_scoped_at_3_0_0_0, 3, 0, 0, 0),
    (l3v007_scoped_at_3_0_0_1, 3, 0, 0, 1),
    (l3v007_scoped_at_3_1_0_0, 3, 1, 0, 0),
    (l3v007_scoped_at_3_1_0_1, 3, 1, 0, 1),
    (l3v007_scoped_at_3_2_0_0, 3, 2, 0, 0),
    (l3v007_scoped_at_3_2_0_1, 3, 2, 0, 1),
);

/// L3V004: the checker's `edges_resolved` is the declarative invariant, at every size.
exact_at_sizes!(edges_resolved, l3v004:
    (l3v004_edges_resolved_at_0_0_0_0, 0, 0, 0, 0),
    (l3v004_edges_resolved_at_0_0_0_1, 0, 0, 0, 1),
    (l3v004_edges_resolved_at_0_0_1_0, 0, 0, 1, 0),
    (l3v004_edges_resolved_at_0_0_1_1, 0, 0, 1, 1),
    (l3v004_edges_resolved_at_0_1_0_0, 0, 1, 0, 0),
    (l3v004_edges_resolved_at_0_1_0_1, 0, 1, 0, 1),
    (l3v004_edges_resolved_at_0_1_1_0, 0, 1, 1, 0),
    (l3v004_edges_resolved_at_0_1_1_1, 0, 1, 1, 1),
    (l3v004_edges_resolved_at_0_2_0_0, 0, 2, 0, 0),
    (l3v004_edges_resolved_at_0_2_0_1, 0, 2, 0, 1),
    (l3v004_edges_resolved_at_0_2_1_0, 0, 2, 1, 0),
    (l3v004_edges_resolved_at_0_2_1_1, 0, 2, 1, 1),
    (l3v004_edges_resolved_at_1_0_0_0, 1, 0, 0, 0),
    (l3v004_edges_resolved_at_1_0_0_1, 1, 0, 0, 1),
    (l3v004_edges_resolved_at_1_0_1_0, 1, 0, 1, 0),
    (l3v004_edges_resolved_at_1_0_1_1, 1, 0, 1, 1),
    (l3v004_edges_resolved_at_1_1_0_0, 1, 1, 0, 0),
    (l3v004_edges_resolved_at_1_1_0_1, 1, 1, 0, 1),
    (l3v004_edges_resolved_at_1_1_1_0, 1, 1, 1, 0),
    (l3v004_edges_resolved_at_1_1_1_1, 1, 1, 1, 1),
    (l3v004_edges_resolved_at_1_2_0_0, 1, 2, 0, 0),
    (l3v004_edges_resolved_at_1_2_0_1, 1, 2, 0, 1),
    (l3v004_edges_resolved_at_1_2_1_0, 1, 2, 1, 0),
    (l3v004_edges_resolved_at_1_2_1_1, 1, 2, 1, 1),
    (l3v004_edges_resolved_at_2_0_0_0, 2, 0, 0, 0),
    (l3v004_edges_resolved_at_2_0_0_1, 2, 0, 0, 1),
    (l3v004_edges_resolved_at_2_0_1_0, 2, 0, 1, 0),
    (l3v004_edges_resolved_at_2_0_1_1, 2, 0, 1, 1),
    (l3v004_edges_resolved_at_2_1_0_0, 2, 1, 0, 0),
    (l3v004_edges_resolved_at_2_1_0_1, 2, 1, 0, 1),
    (l3v004_edges_resolved_at_2_1_1_0, 2, 1, 1, 0),
    (l3v004_edges_resolved_at_2_1_1_1, 2, 1, 1, 1),
    (l3v004_edges_resolved_at_2_2_0_0, 2, 2, 0, 0),
    (l3v004_edges_resolved_at_2_2_0_1, 2, 2, 0, 1),
    (l3v004_edges_resolved_at_2_2_1_0, 2, 2, 1, 0),
    (l3v004_edges_resolved_at_2_2_1_1, 2, 2, 1, 1),
    (l3v004_edges_resolved_at_3_0_0_0, 3, 0, 0, 0),
    (l3v004_edges_resolved_at_3_0_0_1, 3, 0, 0, 1),
    (l3v004_edges_resolved_at_3_0_1_0, 3, 0, 1, 0),
    (l3v004_edges_resolved_at_3_0_1_1, 3, 0, 1, 1),
    (l3v004_edges_resolved_at_3_1_0_0, 3, 1, 0, 0),
    (l3v004_edges_resolved_at_3_1_0_1, 3, 1, 0, 1),
    (l3v004_edges_resolved_at_3_1_1_0, 3, 1, 1, 0),
    (l3v004_edges_resolved_at_3_1_1_1, 3, 1, 1, 1),
    (l3v004_edges_resolved_at_3_2_0_0, 3, 2, 0, 0),
    (l3v004_edges_resolved_at_3_2_0_1, 3, 2, 0, 1),
    (l3v004_edges_resolved_at_3_2_1_0, 3, 2, 1, 0),
    (l3v004_edges_resolved_at_3_2_1_1, 3, 2, 1, 1),
);

/// `wellFormed_ordered`: in the scheduled phase a well-formed program yields
/// the edge-ordering theorem's hypothesis for every edge. Well-formedness
/// includes `ordered`, which alone suffices, so the harness assumes only it.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn a_well_formed_schedule_points_every_edge_forward() {
    let p = graphs::program();
    if checker::ordered(&p) && p.phase == Phase::Scheduled {
        for edge in &p.edges {
            assert!(forward(&p, edge));
        }
    }
}
