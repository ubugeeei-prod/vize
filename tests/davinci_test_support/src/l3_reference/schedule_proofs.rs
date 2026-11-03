//! The laws of the scheduling contract, `schedule.rs`, over the trace
//! semantics, `trace.rs`. They replace `Impeto.ScheduleLaws`. Bound: the
//! graphs of `graphs.rs` (3 ops, 3 regions, 3 edges, 2 effects, ids below 4).

use super::graphs::{self, OPS, id, list, op};
use super::program::{Op, Phase, Program};
use super::schedule::{duplicate_codes, edge_codes, index_of, op_inside, order_codes, precedes};
use super::trace::{Backend, TraceEvent, interpret, run, run_ops};

fn backend() -> Backend {
    if kani::any() {
        Backend::Vdom
    } else {
        Backend::Vapor
    }
}

/// `List.Nodup`: no two positions hold the same id.
fn nodup(ids: &[u32]) -> bool {
    let mut i = 0;
    while i < ids.len() {
        let mut j = i + 1;
        while j < ids.len() {
            if ids.get(i) == ids.get(j) {
                return false;
            }
            j += 1;
        }
        i += 1;
    }
    true
}

/// `Precedes`: both ops execute, and every event of the source precedes
/// every event of the target.
fn holds_before(trace: &[TraceEvent], source: u32, target: u32) -> bool {
    let at = |i: usize, id: u32| trace.get(i).is_some_and(|event| event.op_id == id);
    let mut source_runs = false;
    let mut target_runs = false;
    let mut ordered = true;
    let mut i = 0;
    while i < trace.len() {
        source_runs |= at(i, source);
        target_runs |= at(i, target);
        let mut j = 0;
        while j < trace.len() {
            if at(i, source) && at(j, target) && i >= j {
                ordered = false;
            }
            j += 1;
        }
        i += 1;
    }
    source_runs && target_runs && ordered
}

fn scheduled(program: &Program) -> bool {
    program.phase == Phase::Scheduled
}

/// `getElem?_of_indexOf?`, `indexOf?_of_getElem?`. Bound: 4 ids.
#[kani::proof]
#[kani::unwind(6)]
fn index_of_is_the_position_of_a_distinct_id() {
    let ids = list(4, id);
    let wanted = id();
    if let Some(i) = index_of(&ids, wanted) {
        assert!(ids.get(i) == Some(&wanted));
    }
    let k: usize = kani::any();
    if nodup(&ids) && ids.get(k) == Some(&wanted) {
        assert!(index_of(&ids, wanted) == Some(k));
    }
}

/// `nodup_of_duplicateCodes`. Bound: 2 seen ids, 3 ids.
#[kani::proof]
#[kani::unwind(7)]
fn no_duplicate_code_means_fresh_distinct_ids() {
    let seen = list(2, id);
    let ids = list(3, id);
    if duplicate_codes(&seen, &ids).is_empty() {
        assert!(nodup(&ids));
        assert!(ids.iter().all(|id| !seen.contains(id)));
    }
}

/// `runOps_eq`, `interpret_opId`, `run_opId`: the machine executes ops in
/// list order under both backends. Bound: 3 ops after 1 earlier event.
#[kani::proof]
#[kani::unwind(6)]
fn the_machine_executes_ops_in_list_order() {
    let backend = backend();
    let ops = list(OPS, op);
    let earlier: Vec<TraceEvent> = list(1, || interpret(backend, &op()));
    let mut expected = earlier.clone();
    expected.extend(ops.iter().map(|op| interpret(backend, op)));
    assert!(run_ops(backend, &ops, earlier) == expected);
    for op in &ops {
        assert!(interpret(backend, op).op_id == op.id);
    }
    let mut program = graphs::program();
    program.ops = ops;
    let trace = run(backend, &program);
    assert!(trace.len() == program.ops.len());
    for (event, op) in trace.iter().zip(&program.ops) {
        assert!(event.op_id == op.id);
    }
}

/// `edgeCodes_nil`: a scheduled edge without codes resolves and points forward.
#[kani::proof]
#[kani::unwind(6)]
fn an_accepted_scheduled_edge_points_forward() {
    let program = graphs::program();
    let edge = graphs::edge();
    if edge_codes(&program, &edge).is_empty() && scheduled(&program) {
        let ids = program.ids();
        let source = index_of(&ids, edge.source);
        let target = index_of(&ids, edge.target);
        assert!(matches!((source, target), (Some(i), Some(j)) if i < j));
    }
}

/// `orderCodes_nil`: acceptance means distinct op ids and clean edges.
#[kani::proof]
#[kani::unwind(6)]
fn acceptance_means_distinct_ops_and_clean_edges() {
    let program = graphs::program();
    if order_codes(&program).is_empty() {
        assert!(nodup(&program.ids()));
        for edge in &program.edges {
            assert!(edge_codes(&program, edge).is_empty());
        }
    }
}

/// `accepted_schedule_orders_edges`: validator acceptance implies the
/// semantics orders every state edge, whatever its kind, on both backends.
#[kani::proof]
#[kani::unwind(6)]
fn accepted_schedule_orders_edges() {
    let program = graphs::program();
    if order_codes(&program).is_empty() && scheduled(&program) {
        let trace = run(backend(), &program);
        for edge in &program.edges {
            assert!(holds_before(&trace, edge.source, edge.target));
            assert!(precedes(&trace, edge.source, edge.target));
        }
    }
}

/// `accepted_edges_stay_in_scope`: both endpoints of an accepted
/// effect-scoped edge live inside that effect's region.
#[kani::proof]
#[kani::unwind(6)]
fn accepted_edges_stay_in_scope() {
    let program = graphs::program();
    if order_codes(&program).is_empty() {
        for edge in &program.edges {
            if let Some(scope) = edge.effect {
                let effect = program.effect(scope);
                assert!(effect.is_some_and(|effect| {
                    op_inside(&program, edge.source, effect.region)
                        && op_inside(&program, edge.target, effect.region)
                }));
            }
        }
    }
}

/// A permutation of `ops` (of up to 3), chosen by `choice`. Shorter lists
/// keep the relative order the 3-permutation gives their positions.
fn permute(ops: &[Op], choice: u8) -> Vec<Op> {
    let order: [usize; 3] = match choice % 6 {
        0 => [0, 1, 2],
        1 => [0, 2, 1],
        2 => [1, 0, 2],
        3 => [1, 2, 0],
        4 => [2, 0, 1],
        _ => [2, 1, 0],
    };
    order
        .iter()
        .filter_map(|index| ops.get(*index).copied())
        .collect()
}

fn same_events(left: &[TraceEvent], right: &[TraceEvent]) -> bool {
    let count = |trace: &[TraceEvent], event: &TraceEvent| {
        trace.iter().filter(|other| *other == event).count()
    };
    left.len() == right.len()
        && left
            .iter()
            .all(|event| count(left, event) == count(right, event))
}

/// `accepted_regrouping_preserves_edges`: a regrouping of the same ops (any
/// permutation) and edges that the validator accepts in the scheduled phase,
/// whatever its regions and effects, executes a permutation of the original
/// events and orders every original edge. Bound: 3 ops.
#[kani::proof]
#[kani::unwind(6)]
fn accepted_regrouping_preserves_edges() {
    let original = graphs::program();
    let mut grouped = graphs::program();
    grouped.ops = permute(&original.ops, kani::any());
    grouped.edges = original.edges.clone();
    if order_codes(&grouped).is_empty() && scheduled(&grouped) {
        let backend = backend();
        let trace = run(backend, &grouped);
        assert!(same_events(&trace, &run(backend, &original)));
        for edge in &original.edges {
            assert!(holds_before(&trace, edge.source, edge.target));
        }
    }
}
