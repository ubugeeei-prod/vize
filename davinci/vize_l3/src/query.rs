//! Directed queries over the explicit edges of one immutable L3 IR.
//!
//! A single-use query can scan the edge slice. Repeated queries can share an
//! index over the same IR. Both preserve edge order and parallel edges.

use crate::{
    L3Ir,
    op::{EdgeKind, Op, OpId, StateEdge},
};
use alloc::vec::Vec;

mod check;
mod index;
mod trail;
pub use check::{CertificateError, check_reachability};
pub use index::OpGraph;
pub use trail::{EdgePattern, TrailError, TwoHopMatch, TwoHopQuery, check_two_hop_matches};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Reverse,
}

impl Direction {
    #[inline]
    fn endpoints(self, edge: &StateEdge) -> (OpId, OpId) {
        match self {
            Self::Forward => (edge.from, edge.to),
            Self::Reverse => (edge.to, edge.from),
        }
    }
}

/// The kinds traversed at every step of a query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeKinds(u8);

impl EdgeKinds {
    pub const ALL: Self = Self(7);
    pub const NONE: Self = Self(0);
    pub const fn only(kind: EdgeKind) -> Self {
        Self(1 << kind as u8)
    }
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub const fn contains(self, kind: EdgeKind) -> bool {
        self.0 & Self::only(kind).0 != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReachQuery {
    pub start: OpId,
    pub direction: Direction,
    pub kinds: EdgeKinds,
}

/// Explicit edge ordinals preserve parallel edges with different effects.
/// The first step is the zero-length path at the requested operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReachStep {
    pub op: OpId,
    pub via: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    SizeLimit,
    DuplicateOp(OpId),
    UnknownOp(OpId),
    MissingEndpoint { edge: u32, op: OpId },
}

/// Physical positions index scratch space even when operation IDs are sparse.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Positions {
    sparse: Option<Vec<(OpId, u32)>>,
    len: usize,
}

impl Positions {
    fn read(ops: &[Op], edges: &[StateEdge]) -> Result<Self, GraphError> {
        let len = ops.len();
        if len > u32::MAX as usize || edges.len() > u32::MAX as usize {
            return Err(GraphError::SizeLimit);
        }
        if ops
            .iter()
            .enumerate()
            .all(|(i, op)| i == op.id.index() as usize)
        {
            return Ok(Self { sparse: None, len });
        }
        let mut sorted: Vec<_> = ops
            .iter()
            .enumerate()
            .map(|(i, op)| (op.id, i as u32))
            .collect();
        sorted.sort_unstable();
        if let Some(id) = sorted.windows(2).find_map(|pair| match pair {
            [(a, _), (b, _)] if a == b => Some(*a),
            _ => None,
        }) {
            return Err(GraphError::DuplicateOp(id));
        }
        Ok(Self {
            sparse: Some(sorted),
            len,
        })
    }

    fn get(&self, id: OpId) -> Option<usize> {
        match &self.sparse {
            None => {
                let index = id.index() as usize;
                (index < self.len).then_some(index)
            }
            Some(sorted) => sorted
                .binary_search_by_key(&id, |(op, _)| *op)
                .ok()
                .and_then(|i| sorted.get(i))
                .map(|&(_, p)| p as usize),
        }
    }

    fn endpoint(&self, edge: u32, op: OpId) -> Result<usize, GraphError> {
        self.get(op).ok_or(GraphError::MissingEndpoint { edge, op })
    }
}

/// Scan original edges in emission order. Callers that only need an incidence
/// filter can use this path with zero preprocessing and zero allocations.
pub fn incident<'p>(
    ir: &'p L3Ir<'_>,
    query: ReachQuery,
) -> impl Iterator<Item = (usize, &'p StateEdge)> {
    ir.edges.iter().enumerate().filter(move |(_, edge)| {
        query.kinds.contains(edge.kind) && query.direction.endpoints(edge).0 == query.start
    })
}
