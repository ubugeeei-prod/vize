//! C-23 independent L3 graph checker. It re-checks the graph invariants the
//! TS-27 phase validator enforces from the committed page alone and shares no
//! code with the compiler. Every check is total. Each one carries the
//! declarative statement of its invariant (Lean `Impeto.Checker.WellFormed`)
//! as its contract, and `creusot.rs` proves the checker exact against them.

#[cfg(creusot)]
use creusot_std::prelude::*;

use super::program::{EffectScope, Phase, Program, Row, Span, StateEdge, index_of};
#[cfg(creusot)]
use super::program::{distinct, distinct_before, first_at};
use super::schedule::{
    L3V001, L3V002, L3V003, L3V004, L3V005, L3V006, L3V007, L3V008, is_or_descendant, op_index,
    op_inside, region_parent,
};
#[cfg(creusot)]
use super::schedule::{descends, inside};

/// `outer` covers `inner`.
#[cfg(creusot)]
#[logic(open)]
pub fn within(outer: Span, inner: Span) -> bool {
    pearlite! { outer.start@ <= inner.start@ && inner.stop@ <= outer.stop@ }
}

#[cfg_attr(creusot, ensures(result == within(outer, inner)))]
#[must_use]
pub fn contains(outer: Span, inner: Span) -> bool {
    outer.start <= inner.start && inner.stop <= outer.stop
}

/// The chain of first-match parents from `region` reaches a parentless
/// region within `fuel` lookups (Lean `reachesTop`).
#[cfg(creusot)]
#[logic(open)]
#[variant(fuel)]
pub fn chain_ends(p: Program, fuel: Int, region: u32) -> bool {
    pearlite! {
        fuel > 0 && exists<i: Int> first_at(p.regions@, region, i)
            && match p.regions@[i].parent {
                None => true,
                Some(parent) => chain_ends(p, fuel - 1, parent),
            }
    }
}

/// The parent chain from `region` reaches a parentless region, visiting at
/// most one more region than the program has.
#[cfg_attr(creusot, ensures(result == chain_ends(*program, program.regions@.len() + 1, region)))]
#[must_use]
pub fn reaches_top(program: &Program, region: u32) -> bool {
    let mut current = region;
    #[cfg_attr(creusot, invariant(chain_ends(*program, program.regions@.len() + 1, region)
        == chain_ends(*program, program.regions@.len() + 1 - produced.len(), current)))]
    for _ in 0..=program.regions.len() {
        match region_parent(program, current) {
            Some(Some(parent)) => current = parent,
            Some(None) => return true,
            None => return false,
        }
    }
    false
}

/// The first op with the edge's source precedes the first op with its target.
#[cfg(creusot)]
#[logic(open)]
pub fn points_forward(p: Program, edge: StateEdge) -> bool {
    pearlite! {
        exists<i: Int, j: Int> first_at(p.ops@, edge.source, i)
            && first_at(p.ops@, edge.target, j) && i < j
    }
}

#[cfg_attr(creusot, ensures(result == points_forward(*program, *edge)))]
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

/// Some row satisfies `holds`.
#[cfg_attr(creusot, requires(forall<i: Int> 0 <= i && i < rows@.len()
    ==> holds.precondition((rows@[i],))))]
#[cfg_attr(creusot, ensures(result ==> exists<i: Int> 0 <= i && i < rows@.len()
    && holds.postcondition((rows@[i],), true)))]
#[cfg_attr(creusot, ensures(!result ==> forall<i: Int> 0 <= i && i < rows@.len()
    ==> holds.postcondition((rows@[i],), false)))]
fn any<T: Copy>(rows: &[T], holds: impl Fn(T) -> bool) -> bool {
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < produced.len()
        ==> holds.postcondition((rows@[i],), false)))]
    for i in 0..rows.len() {
        if let Some(row) = rows.get(i)
            && holds(*row)
        {
            return true;
        }
    }
    false
}

/// Every row satisfies `holds`.
#[cfg_attr(creusot, requires(forall<i: Int> 0 <= i && i < rows@.len()
    ==> holds.precondition((rows@[i],))))]
#[cfg_attr(creusot, ensures(result ==> forall<i: Int> 0 <= i && i < rows@.len()
    ==> holds.postcondition((rows@[i],), true)))]
#[cfg_attr(creusot, ensures(!result ==> exists<i: Int> 0 <= i && i < rows@.len()
    && holds.postcondition((rows@[i],), false)))]
fn all<T: Copy>(rows: &[T], holds: impl Fn(T) -> bool) -> bool {
    #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < produced.len()
        ==> holds.postcondition((rows@[i],), true)))]
    for i in 0..rows.len() {
        if let Some(row) = rows.get(i)
            && !holds(*row)
        {
            return false;
        }
    }
    true
}

/// No two rows share an id: each row's id first occurs at the row itself.
#[cfg_attr(creusot, ensures(result == distinct(rows@)))]
fn unique<T: Row>(rows: &[T]) -> bool {
    #[cfg_attr(creusot, invariant(distinct_before(rows@, produced.len())))]
    for i in 0..rows.len() {
        if let Some(row) = rows.get(i)
            && index_of(rows, row.key()) != Some(i)
        {
            return false;
        }
    }
    true
}

/// L3V001: op, region and effect ids are unique.
#[cfg(creusot)]
#[logic(open)]
pub fn ids_unique(p: Program) -> bool {
    pearlite! { distinct(p.ops@) && distinct(p.regions@) && distinct(p.effects@) }
}

#[cfg_attr(creusot, ensures(result == ids_unique(*p)))]
#[must_use]
pub fn unique_ids(p: &Program) -> bool {
    unique(&p.ops) && unique(&p.regions) && unique(&p.effects)
}

/// L3V002: region 0 exists and is the parentless, ownerless root.
#[cfg(creusot)]
#[logic(open)]
pub fn has_root(p: Program) -> bool {
    pearlite! {
        exists<i: Int> 0 <= i && i < p.regions@.len() && p.regions@[i].id == 0u32
            && p.regions@[i].parent == None && p.regions@[i].owner == None
    }
}

#[cfg_attr(creusot, ensures(result == has_root(*p)))]
#[must_use]
pub fn rooted(p: &Program) -> bool {
    any(&p.regions, |r| {
        r.id == 0 && r.parent.is_none() && r.owner.is_none()
    })
}

/// L3V005: every other region names a resolvable parent and an owner op in it.
#[cfg(creusot)]
#[logic(open)]
pub fn parents_resolve(p: Program) -> bool {
    pearlite! {
        forall<i: Int> 0 <= i && i < p.regions@.len() ==> p.regions@[i].id == 0u32 || (
            (exists<j: Int> 0 <= j && j < p.regions@.len()
                && p.regions@[i].parent == Some(p.regions@[j].id))
            && exists<k: Int> 0 <= k && k < p.ops@.len()
                && p.regions@[i].owner == Some(p.ops@[k].id)
                && p.regions@[i].parent == Some(p.ops@[k].region))
    }
}

#[cfg_attr(creusot, ensures(result == parents_resolve(*p)))]
#[must_use]
pub fn resolved(p: &Program) -> bool {
    all(&p.regions, |r| {
        r.id == 0
            || (any(&p.regions, |q| r.parent == Some(q.id))
                && any(&p.ops, |o| {
                    r.owner == Some(o.id) && r.parent == Some(o.region)
                }))
    })
}

/// L3V006: spans nest (region in parent, region in owner, op in region) and
/// every parent chain is finite.
#[cfg(creusot)]
#[logic(open)]
pub fn spans_nest(p: Program) -> bool {
    pearlite! {
        (forall<i: Int, j: Int> 0 <= i && i < p.regions@.len() && 0 <= j && j < p.regions@.len()
            && p.regions@[i].parent == Some(p.regions@[j].id)
            ==> within(p.regions@[j].span, p.regions@[i].span))
        && (forall<i: Int, k: Int> 0 <= i && i < p.regions@.len() && 0 <= k && k < p.ops@.len()
            && p.regions@[i].owner == Some(p.ops@[k].id)
            ==> within(p.ops@[k].span, p.regions@[i].span))
        && (forall<k: Int, i: Int> 0 <= k && k < p.ops@.len() && 0 <= i && i < p.regions@.len()
            && p.regions@[i].id == p.ops@[k].region
            ==> within(p.regions@[i].span, p.ops@[k].span))
        && forall<i: Int> 0 <= i && i < p.regions@.len()
            ==> chain_ends(p, p.regions@.len() + 1, p.regions@[i].id)
    }
}

#[cfg_attr(creusot, ensures(result == spans_nest(*p)))]
#[must_use]
pub fn nested(p: &Program) -> bool {
    all(&p.regions, |r| {
        all(&p.regions, |q| {
            r.parent != Some(q.id) || contains(q.span, r.span)
        }) && all(&p.ops, |o| {
            r.owner != Some(o.id) || contains(o.span, r.span)
        }) && reaches_top(p, r.id)
    }) && all(&p.ops, |o| {
        all(&p.regions, |r| r.id != o.region || contains(r.span, o.span))
    })
}

/// L3V003: ops live in existing regions.
#[cfg(creusot)]
#[logic(open)]
pub fn ops_live(p: Program) -> bool {
    pearlite! {
        forall<k: Int> 0 <= k && k < p.ops@.len() ==> exists<i: Int> 0 <= i && i < p.regions@.len()
            && p.regions@[i].id == p.ops@[k].region
    }
}

#[cfg_attr(creusot, ensures(result == ops_live(*p)))]
#[must_use]
pub fn ops_resolved(p: &Program) -> bool {
    all(&p.ops, |o| any(&p.regions, |r| r.id == o.region))
}

/// L3V007, first half: ops name existing effects.
#[cfg(creusot)]
#[logic(open)]
pub fn effects_named(p: Program) -> bool {
    pearlite! {
        forall<k: Int> 0 <= k && k < p.ops@.len() ==> match p.ops@[k].effect {
            None => true,
            Some(e) => exists<s: Int> 0 <= s && s < p.effects@.len() && p.effects@[s].id == e,
        }
    }
}

/// The owner op of `effect` lives in the effect's region or below it.
#[cfg(creusot)]
#[logic(open)]
pub fn owned(p: Program, effect: EffectScope) -> bool {
    pearlite! {
        exists<k: Int> 0 <= k && k < p.ops@.len() && p.ops@[k].id == effect.owner
            && descends(p, p.regions@.len() + 1, p.ops@[k].region, effect.region)
    }
}

/// The span of `effect` nests in a region with the effect's region id.
#[cfg(creusot)]
#[logic(open)]
pub fn spanned(p: Program, effect: EffectScope) -> bool {
    pearlite! {
        exists<i: Int> 0 <= i && i < p.regions@.len() && p.regions@[i].id == effect.region
            && within(p.regions@[i].span, effect.span)
    }
}

/// L3V007, second half: an effect's owner lives in its region (or below)
/// and its span nests in that region.
#[cfg(creusot)]
#[logic(open)]
pub fn effects_placed(p: Program) -> bool {
    pearlite! {
        forall<s: Int> 0 <= s && s < p.effects@.len()
            ==> owned(p, p.effects@[s]) && spanned(p, p.effects@[s])
    }
}

/// L3V007: ops name existing effects; an effect's owner lives in its region
/// (or below) and its span nests in that region.
#[cfg(creusot)]
#[logic(open)]
pub fn effects_scoped(p: Program) -> bool {
    pearlite! { effects_named(p) && effects_placed(p) }
}

#[cfg_attr(creusot, ensures(result == effects_named(*p)))]
fn named(p: &Program) -> bool {
    all(&p.ops, |o| match o.effect {
        Some(e) => any(&p.effects, |s| s.id == e),
        None => true,
    })
}

#[cfg_attr(creusot, ensures(result == owned(*p, effect)))]
fn owner_inside(p: &Program, effect: EffectScope) -> bool {
    any(&p.ops, |o| {
        o.id == effect.owner && is_or_descendant(p, o.region, effect.region)
    })
}

#[cfg_attr(creusot, ensures(result == spanned(*p, effect)))]
fn span_inside(p: &Program, effect: EffectScope) -> bool {
    any(&p.regions, |r| {
        r.id == effect.region && contains(r.span, effect.span)
    })
}

#[cfg_attr(creusot, ensures(result == effects_placed(*p)))]
fn placed(p: &Program) -> bool {
    all(&p.effects, |s| owner_inside(p, s) && span_inside(p, s))
}

#[cfg_attr(creusot, ensures(result == effects_scoped(*p)))]
#[must_use]
pub fn scoped(p: &Program) -> bool {
    named(p) && placed(p)
}

/// L3V004 / L3V007: edges resolve and effect-scoped edges stay in scope.
#[cfg(creusot)]
#[logic(open)]
pub fn edges_stay(p: Program) -> bool {
    pearlite! {
        forall<e: Int> 0 <= e && e < p.edges@.len() ==>
            (exists<k: Int> 0 <= k && k < p.ops@.len() && p.ops@[k].id == p.edges@[e].source)
            && (exists<k: Int> 0 <= k && k < p.ops@.len() && p.ops@[k].id == p.edges@[e].target)
            && match p.edges@[e].effect {
                None => true,
                Some(scope) => exists<s: Int> 0 <= s && s < p.effects@.len()
                    && p.effects@[s].id == scope
                    && inside(p, p.edges@[e].source, p.effects@[s].region)
                    && inside(p, p.edges@[e].target, p.effects@[s].region),
            }
    }
}

#[cfg_attr(creusot, ensures(result == edges_stay(*p)))]
#[must_use]
pub fn edges_resolved(p: &Program) -> bool {
    all(&p.edges, |e| {
        any(&p.ops, |o| o.id == e.source)
            && any(&p.ops, |o| o.id == e.target)
            && match e.effect {
                Some(scope) => any(&p.effects, |s| {
                    s.id == scope
                        && op_inside(p, e.source, s.region)
                        && op_inside(p, e.target, s.region)
                }),
                None => true,
            }
    })
}

/// L3V008: in the scheduled phase every edge points strictly forward.
#[cfg(creusot)]
#[logic(open)]
pub fn edges_forward(p: Program) -> bool {
    pearlite! {
        p.phase != Phase::Scheduled
            || forall<e: Int> 0 <= e && e < p.edges@.len() ==> points_forward(p, p.edges@[e])
    }
}

#[cfg_attr(creusot, ensures(result == edges_forward(*p)))]
#[must_use]
pub fn ordered(p: &Program) -> bool {
    !matches!(p.phase, Phase::Scheduled) || all(&p.edges, |e| forward(p, &e))
}

/// Lean `WellFormed`: every invariant holds.
#[cfg(creusot)]
#[logic(open)]
pub fn well_formed(p: Program) -> bool {
    pearlite! {
        ids_unique(p) && has_root(p) && parents_resolve(p) && spans_nest(p)
            && ops_live(p) && effects_scoped(p) && edges_stay(p) && edges_forward(p)
    }
}

/// Each invariant with its code, in validator order.
#[cfg_attr(creusot, ensures(result@[0] == (ids_unique(*p), L3V001)))]
#[cfg_attr(creusot, ensures(result@[1] == (has_root(*p), L3V002)))]
#[cfg_attr(creusot, ensures(result@[2] == (parents_resolve(*p), L3V005)))]
#[cfg_attr(creusot, ensures(result@[3] == (spans_nest(*p), L3V006)))]
#[cfg_attr(creusot, ensures(result@[4] == (ops_live(*p), L3V003)))]
#[cfg_attr(creusot, ensures(result@[5] == (effects_scoped(*p), L3V007)))]
#[cfg_attr(creusot, ensures(result@[6] == (edges_stay(*p), L3V004)))]
#[cfg_attr(creusot, ensures(result@[7] == (edges_forward(*p), L3V008)))]
#[must_use]
pub fn checks(p: &Program) -> [(bool, &'static str); 8] {
    [
        (unique_ids(p), L3V001),
        (rooted(p), L3V002),
        (resolved(p), L3V005),
        (nested(p), L3V006),
        (ops_resolved(p), L3V003),
        (scoped(p), L3V007),
        (edges_resolved(p), L3V004),
        (ordered(p), L3V008),
    ]
}

/// One code per violated invariant, in validator order: none exactly for a
/// well-formed program (Lean `violations_nil_iff`).
#[cfg_attr(creusot, ensures((result@.len() == 0) == well_formed(*p)))]
#[must_use]
pub fn violations(p: &Program) -> Vec<&'static str> {
    let checks = checks(p);
    let mut codes = Vec::new();
    #[cfg_attr(creusot, invariant((codes@.len() == 0)
        == forall<k: Int> 0 <= k && k < produced.len() ==> checks@[k].0))]
    for k in 0..checks.len() {
        if let Some(&(holds, code)) = checks.get(k)
            && !holds
        {
            codes.push(code);
        }
    }
    codes
}
