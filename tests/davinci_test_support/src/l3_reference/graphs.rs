//! Bounded symbolic L3 graphs for the harnesses.

use super::program::{EdgeKind, EffectScope, Op, OpKind, Phase, Program, Region, Span, StateEdge};

/// Ids range over `0..IDS`: one more than the largest table, so every
/// harness also sees ids that resolve to nothing.
pub const IDS: u32 = 4;
pub const OPS: usize = 3;
pub const REGIONS: usize = 3;
pub const EDGES: usize = 3;
pub const EFFECTS: usize = 2;

pub fn id() -> u32 {
    let id: u32 = kani::any();
    kani::assume(id < IDS);
    id
}

pub fn maybe_id() -> Option<u32> {
    if kani::any() { Some(id()) } else { None }
}

/// Span ends are unconstrained: containment is a pure order comparison.
pub fn span() -> Span {
    Span {
        start: kani::any(),
        stop: kani::any(),
    }
}

/// Between zero and `max` items.
pub fn list<T>(max: usize, item: impl Fn() -> T) -> Vec<T> {
    let len: usize = kani::any();
    kani::assume(len <= max);
    let mut out = Vec::new();
    for index in 0..max {
        if index < len {
            out.push(item());
        }
    }
    out
}

pub fn phase() -> Phase {
    match kani::any::<u8>() % 3 {
        0 => Phase::Built,
        1 => Phase::Partitioned,
        _ => Phase::Scheduled,
    }
}

pub fn op() -> Op {
    let kind: usize = kani::any();
    kani::assume(kind < OpKind::ALL.len());
    Op {
        id: id(),
        kind: OpKind::ALL.get(kind).copied().unwrap_or(OpKind::SetProp),
        region: id(),
        effect: maybe_id(),
        span: span(),
    }
}

pub fn edge() -> StateEdge {
    StateEdge {
        source: id(),
        target: id(),
        kind: match kani::any::<u8>() % 3 {
            0 => EdgeKind::DomOrder,
            1 => EdgeKind::EffectOrder,
            _ => EdgeKind::DataDependency,
        },
        effect: maybe_id(),
    }
}

pub fn program() -> Program {
    Program {
        phase: phase(),
        regions: list(REGIONS, || Region {
            id: id(),
            parent: maybe_id(),
            owner: maybe_id(),
            span: span(),
        }),
        ops: list(OPS, op),
        edges: list(EDGES, edge),
        effects: list(EFFECTS, || EffectScope {
            id: id(),
            owner: id(),
            region: id(),
            span: span(),
        }),
    }
}
