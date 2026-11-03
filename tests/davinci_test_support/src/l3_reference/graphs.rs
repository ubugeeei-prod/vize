//! Bounded symbolic L3 graphs for the harnesses.

use super::program::{EdgeKind, EffectScope, Op, OpKind, Phase, Program, Region, Span, StateEdge};

/// Ids range over `0..IDS`, so short or duplicated tables leave ids that
/// resolve to nothing, and two regions already form parent cycles. One edge
/// suffices: every edge law is a conjunction over edges, and no edge reads
/// another.
pub const IDS: u32 = 3;
pub const OPS: usize = 3;
pub const REGIONS: usize = 2;
pub const EDGES: usize = 1;
pub const EFFECTS: usize = 1;

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

/// The first `len` items, without growing or cloning a vector of symbolic
/// length (both cost CBMC an order of magnitude).
pub fn cut<T, const N: usize>(items: [T; N], len: usize) -> Vec<T> {
    let mut out = Vec::from(items);
    out.truncate(len);
    out
}

/// Between zero and `N` items. A fixed array cut to a symbolic length:
/// growing a vector push by push costs CBMC an order of magnitude more.
pub fn list<T, const N: usize>(item: impl Fn() -> T) -> Vec<T> {
    let len: usize = kani::any();
    kani::assume(len <= N);
    cut(core::array::from_fn::<T, N, _>(|_| item()), len)
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

/// A graph for the scheduling laws, which read regions and effects only by
/// id lookup. Their tables have fixed sizes, so CBMC unrolls every scan
/// exactly; a slot whose id is `IDS` resolves for no lookup, so the fixed
/// tables cover every shorter one. One edge: every scheduling law is a
/// conjunction over edges, and no edge reads another.
pub fn schedule_graph() -> Program {
    let slot = || {
        let id: u32 = kani::any();
        kani::assume(id <= IDS);
        id
    };
    Program {
        phase: phase(),
        regions: Vec::from(core::array::from_fn::<_, REGIONS, _>(|_| Region {
            id: slot(),
            parent: maybe_id(),
            owner: maybe_id(),
            span: span(),
        })),
        ops: list::<_, OPS>(op),
        edges: vec![edge()],
        effects: vec![EffectScope {
            id: slot(),
            owner: id(),
            region: id(),
            span: span(),
        }],
    }
}

pub fn program() -> Program {
    Program {
        phase: phase(),
        regions: list::<_, REGIONS>(|| Region {
            id: id(),
            parent: maybe_id(),
            owner: maybe_id(),
            span: span(),
        }),
        ops: list::<_, OPS>(op),
        edges: list::<_, EDGES>(edge),
        effects: list::<_, EFFECTS>(|| EffectScope {
            id: id(),
            owner: id(),
            region: id(),
            span: span(),
        }),
    }
}

/// A graph with exactly the given table sizes.
pub fn fixed<const O: usize, const R: usize, const E: usize, const F: usize>() -> Program {
    Program {
        phase: phase(),
        regions: Vec::from(core::array::from_fn::<_, R, _>(|_| Region {
            id: id(),
            parent: maybe_id(),
            owner: maybe_id(),
            span: span(),
        })),
        ops: Vec::from(core::array::from_fn::<_, O, _>(|_| op())),
        edges: Vec::from(core::array::from_fn::<_, E, _>(|_| edge())),
        effects: Vec::from(core::array::from_fn::<_, F, _>(|_| EffectScope {
            id: id(),
            owner: id(),
            region: id(),
            span: span(),
        })),
    }
}
