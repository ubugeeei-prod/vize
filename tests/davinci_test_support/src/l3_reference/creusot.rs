//! Creusot proves the laws of the independent L3 reference on its Rust
//! source: the P3-15 scheduling contract over the P3-4 trace semantics, and
//! the C-23 graph checker. This file is the library root of
//! `tests/formal/creusot/reference` and is not part of
//! `davinci_test_support`'s module tree, as `kani.rs` beside it is not: the
//! modules below are the ones `l3_reference.rs` compiles, found by ordinary
//! module discovery.
//!
//! Each check carries its law as a contract on the shipped function: the
//! lookups return the first row with an id, `duplicates`, `edge_codes`,
//! `order_codes` and `accepts` state the validator's codes, `run_ops` and
//! `run` the trace semantics, and every checker invariant equals its
//! declarative statement. The lemmas below are the laws that compose those
//! contracts. They replace the bounded Kani harnesses of the same names
//! (docs/davinci/decisions/2026-11-17-creusot-inventory.md) and hold for
//! programs of any size. The incremental update machine stays on Kani.

pub mod checker;
pub mod program;
pub mod schedule;
pub mod trace;

use creusot_std::prelude::*;

use checker::{edges_forward, points_forward, well_formed};
use program::{Phase, Program, StateEdge, first_at};
use schedule::{accepted, clean, happens_before, inside};
use trace::{Backend, TraceEvent, observe};

/// Lean `edgeCodes_nil` (Kani `an_accepted_scheduled_edge_points_forward`):
/// a scheduled edge without codes resolves and points forward. `edge_codes`
/// has no code exactly when the edge is `clean`.
#[logic]
#[requires(clean(program, edge) && program.phase == Phase::Scheduled)]
#[ensures(exists<i: Int, j: Int> first_at(program.ops@, edge.source, i)
    && first_at(program.ops@, edge.target, j) && i < j)]
pub fn an_accepted_scheduled_edge_points_forward(program: Program, edge: StateEdge) {}

/// Lean `accepted_schedule_orders_edges` (Kani of the same name): validator
/// acceptance in the scheduled phase orders every state edge, whatever its
/// kind, in the trace of either backend. `run` returns exactly that trace.
#[logic]
#[requires(accepted(program) && program.phase == Phase::Scheduled)]
#[requires(trace.len() == program.ops@.len())]
#[requires(forall<i: Int> 0 <= i && i < trace.len() ==> trace[i] == observe(backend, program.ops@[i]))]
#[ensures(forall<k: Int> 0 <= k && k < program.edges@.len()
    ==> happens_before(trace, program.edges@[k].source, program.edges@[k].target))]
pub fn accepted_schedule_orders_edges(program: Program, backend: Backend, trace: Seq<TraceEvent>) {}

/// Lean `accepted_edges_stay_in_scope` (Kani of the same name): both
/// endpoints of an accepted effect-scoped edge live inside the region of the
/// first effect with its id.
#[logic]
#[requires(accepted(program))]
#[ensures(forall<k: Int, scope: u32> 0 <= k && k < program.edges@.len()
    && program.edges@[k].effect == Some(scope)
    ==> exists<s: Int> first_at(program.effects@, scope, s)
        && inside(program, program.edges@[k].source, program.effects@[s].region)
        && inside(program, program.edges@[k].target, program.effects@[s].region))]
pub fn accepted_edges_stay_in_scope(program: Program) {}

/// Lean `accepted_regrouping_preserves_edges` (Kani of the same name): a
/// regrouping of the same ops by any permutation, with the same edges, that
/// the validator accepts in the scheduled phase, whatever its regions and
/// effects, executes the original events permuted by the regrouping's own
/// permutation and orders every original edge.
#[logic]
#[requires(grouped.ops@.len() == original.ops@.len() && order.len() == original.ops@.len())]
#[requires(forall<k: Int> 0 <= k && k < order.len() ==> 0 <= order[k] && order[k] < order.len())]
#[requires(forall<a: Int, b: Int> 0 <= a && a < order.len() && 0 <= b && b < order.len()
    && order[a] == order[b] ==> a == b)]
#[requires(forall<k: Int> 0 <= k && k < order.len() ==> grouped.ops@[k] == original.ops@[order[k]])]
#[requires(grouped.edges@ == original.edges@)]
#[requires(accepted(grouped) && grouped.phase == Phase::Scheduled)]
#[requires(before.len() == original.ops@.len())]
#[requires(forall<i: Int> 0 <= i && i < before.len() ==> before[i] == observe(backend, original.ops@[i]))]
#[requires(after.len() == grouped.ops@.len())]
#[requires(forall<i: Int> 0 <= i && i < after.len() ==> after[i] == observe(backend, grouped.ops@[i]))]
#[ensures(after.len() == before.len())]
#[ensures(forall<k: Int> 0 <= k && k < after.len() ==> after[k] == before[order[k]])]
#[ensures(forall<k: Int> 0 <= k && k < original.edges@.len()
    ==> happens_before(after, original.edges@[k].source, original.edges@[k].target))]
pub fn accepted_regrouping_preserves_edges(
    original: Program,
    grouped: Program,
    order: Seq<Int>,
    backend: Backend,
    before: Seq<TraceEvent>,
    after: Seq<TraceEvent>,
) {
    accepted_schedule_orders_edges(grouped, backend, after)
}

/// Lean `wellFormed_ordered` (Kani
/// `a_well_formed_schedule_points_every_edge_forward`): in the scheduled
/// phase a well-formed program gives the edge-ordering theorem its
/// hypothesis for every edge. `ordered` and `forward` return exactly
/// `edges_forward` and `points_forward`.
#[logic]
#[requires(well_formed(program) && program.phase == Phase::Scheduled)]
#[ensures(edges_forward(program))]
#[ensures(forall<k: Int> 0 <= k && k < program.edges@.len()
    ==> points_forward(program, program.edges@[k]))]
pub fn a_well_formed_schedule_points_every_edge_forward(program: Program) {}
