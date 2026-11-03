//! The laws of the scheduling contract, `schedule.rs`, over the trace
//! semantics, `trace.rs`. They replace `Impeto.ScheduleLaws`. Bound:
//! `graphs::schedule_graph` (up to 3 ops, 2 region and 1 effect slots, 1 edge,
//! ids below 3).

use super::graphs::{self, EDGES, OPS, cut, id, list, op};
use super::program::{Op, Phase, Program, StateEdge};
use super::schedule::{
    accepts, duplicate_codes, edge_codes, index_of, op_inside, order_codes, precedes,
};
use super::trace::{Backend, TraceEvent, interpret, run, run_ops};

fn backend() -> Backend {
    if kani::any() {
        Backend::Vdom
    } else {
        Backend::Vapor
    }
}

/// `List.Nodup`: no two positions hold the same id.
fn nodup<T>(items: &[T], id: impl Fn(&T) -> u32) -> bool {
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
#[kani::unwind(5)]
#[kani::solver(cadical)]
fn index_of_is_the_position_of_a_distinct_id() {
    let ids = list::<_, 4>(id);
    let wanted = id();
    if let Some(i) = index_of(ids.iter().copied(), wanted) {
        assert!(ids.get(i) == Some(&wanted));
    }
    let k: usize = kani::any();
    if nodup(&ids, |id| *id) && ids.get(k) == Some(&wanted) {
        assert!(index_of(ids.iter().copied(), wanted) == Some(k));
    }
}

/// `nodup_of_duplicateCodes`. Bound: 2 seen ids, 3 ids.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn no_duplicate_code_means_fresh_distinct_ids() {
    let seen = list::<_, 2>(id);
    let ids = list::<_, 3>(id);
    if duplicate_codes(&seen, &ids, |id| *id).next().is_none() {
        assert!(nodup(&ids, |id| *id));
        assert!(ids.iter().all(|id| seen.iter().all(|s| s != id)));
    }
}

/// `runOps_eq`, `interpret_opId`: from an earlier trace, the machine
/// appends one observation per op, in list order, under both backends.
/// Bound: 2 ops after exactly 1 earlier event (the empty trace is
/// `trace_positions_are_op_positions`).
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn the_machine_appends_each_op_in_list_order() {
    let backend = backend();
    let ops = list::<_, 2>(op);
    let earlier = vec![interpret(backend, &op())];
    let mut expected = earlier.clone();
    for op in &ops {
        expected.push(interpret(backend, op));
        assert!(interpret(backend, op).op_id == op.id);
    }
    assert!(run_ops(backend, &ops, earlier) == expected);
}

/// `run_opId`: trace positions are op positions. Bound: 3 ops.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn trace_positions_are_op_positions() {
    let program = Program {
        ops: list::<_, OPS>(op),
        ..Program::empty()
    };
    let trace = run(backend(), &program);
    assert!(trace.len() == program.ops.len());
    for (event, op) in trace.iter().zip(&program.ops) {
        assert!(event.op_id == op.id);
    }
}

/// `edgeCodes_nil`: a scheduled edge without codes resolves and points forward.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn an_accepted_scheduled_edge_points_forward() {
    let program = graphs::schedule_graph();
    let edge = graphs::edge();
    if matches!(edge_codes(&program, &edge), [None, None, None, None]) && scheduled(&program) {
        let ids = || program.ops.iter().map(|op| op.id);
        let source = index_of(ids(), edge.source);
        let target = index_of(ids(), edge.target);
        assert!(matches!((source, target), (Some(i), Some(j)) if i < j));
    }
}

/// `accepts`, which the laws read, is exactly an empty `order_codes`. The
/// tie is iterator structure, not graph content, so the graph is minimal: 2
/// ops and 1 edge reach a duplicate and every one of an edge's four codes.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn acceptance_is_an_empty_verdict() {
    let program = Program {
        phase: graphs::phase(),
        ops: list::<_, 2>(op),
        edges: vec![graphs::edge()],
        ..Program::empty()
    };
    assert!(accepts(&program) == order_codes(&program).is_empty());
}

/// `orderCodes_nil`: acceptance means distinct op ids and clean edges.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn acceptance_means_distinct_ops_and_clean_edges() {
    let program = graphs::schedule_graph();
    if accepts(&program) {
        assert!(nodup(&program.ops, |op| op.id));
        for edge in &program.edges {
            assert!(matches!(
                edge_codes(&program, edge),
                [None, None, None, None]
            ));
        }
    }
}

/// `accepted_schedule_orders_edges`: validator acceptance implies the
/// semantics orders every state edge, whatever its kind, on both backends.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn accepted_schedule_orders_edges() {
    let program = graphs::schedule_graph();
    if accepts(&program) && scheduled(&program) {
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
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn accepted_edges_stay_in_scope() {
    let program = graphs::schedule_graph();
    if accepts(&program) {
        for edge in &program.edges {
            if let Some(scope) = edge.effect {
                let effect = program.effects.iter().find(|effect| effect.id == scope);
                assert!(effect.is_some_and(|effect| {
                    op_inside(&program, edge.source, effect.region)
                        && op_inside(&program, edge.target, effect.region)
                }));
            }
        }
    }
}

/// A permutation of `0..3`, chosen by `choice`.
fn permutation(choice: u8) -> [usize; 3] {
    match choice % 6 {
        0 => [0, 1, 2],
        1 => [0, 2, 1],
        2 => [1, 0, 2],
        3 => [1, 2, 0],
        4 => [2, 0, 1],
        _ => [2, 1, 0],
    }
}

/// `accepted_regrouping_preserves_edges`: a regrouping of the same ops (any
/// permutation) and edges that the validator accepts in the scheduled phase,
/// whatever its regions and effects, executes a permutation of the original
/// events (witnessed by the regrouping's own permutation) and orders every
/// original edge. Bound: 3 ops.
#[kani::proof]
#[kani::unwind(4)]
#[kani::solver(cadical)]
fn accepted_regrouping_preserves_edges() {
    let ops: [Op; 3] = [op(), op(), op()];
    let edges: [StateEdge; EDGES] = core::array::from_fn(|_| graphs::edge());
    let (len, edge_count): (usize, usize) = (kani::any(), kani::any());
    kani::assume(len <= 3 && edge_count <= EDGES);
    // The first `len` positions permute `0..len`: every permutation of every
    // list of up to 3 ops.
    let order = permutation(kani::any());
    kani::assume(order.iter().take(len).all(|index| *index < len));
    let mut original = graphs::schedule_graph();
    original.ops = cut(ops, len);
    original.edges = cut(edges, edge_count);
    let mut grouped = graphs::schedule_graph();
    grouped.ops = cut(
        order.map(|index| ops.get(index).copied().unwrap_or(ops[0])),
        len,
    );
    grouped.edges = cut(edges, edge_count);
    if accepts(&grouped) && scheduled(&grouped) {
        let backend = backend();
        let trace = run(backend, &grouped);
        let before = run(backend, &original);
        assert!(trace.len() == before.len());
        for (event, index) in trace.iter().zip(order) {
            assert!(Some(event) == before.get(index));
        }
        for edge in &original.edges {
            assert!(holds_before(&trace, edge.source, edge.target));
        }
    }
}
