//! Check a reachability proof against the original edge table without the
//! producer's adjacency index, traversal or queue.
use super::{GraphError, L3Ir, OpId, Positions, ReachQuery, ReachStep};
use crate::op::{Op, StateEdge};
use alloc::vec;

mod rule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateError {
    Graph(GraphError),
    Root,
    Duplicate(OpId),
    Witness { op: OpId },
    OpenEdge(u32),
}

impl From<GraphError> for CertificateError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

/// Every listed operation has a path back to the seed through earlier rows.
/// Closure under every matching original edge proves that no reachable
/// operation was omitted. The checker accepts other valid witness orders.
pub fn check_reachability(
    ir: &L3Ir<'_>,
    query: ReachQuery,
    steps: &[ReachStep],
) -> Result<(), CertificateError> {
    check_rows(&ir.ops, &ir.edges, query, steps)
}

#[inline(never)]
pub(super) fn check_rows(
    ops: &[Op],
    edges: &[StateEdge],
    query: ReachQuery,
    steps: &[ReachStep],
) -> Result<(), CertificateError> {
    let positions = Positions::read(ops, edges)?;
    include!("check/witness.rs")
}

pub(super) fn check_with_positions(
    positions: &Positions,
    ops: &[Op],
    edges: &[StateEdge],
    query: ReachQuery,
    steps: &[ReachStep],
) -> Result<(), CertificateError> {
    include!("check/witness.rs")
}
