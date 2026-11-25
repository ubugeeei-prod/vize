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
    let endpoints_validated = false;
    include!("check/witness.rs")
}

pub(super) fn check_with_positions(
    positions: &Positions,
    ops: &[Op],
    edges: &[StateEdge],
    query: ReachQuery,
    steps: &[ReachStep],
) -> Result<(), CertificateError> {
    let endpoints_validated = true;
    include!("check/witness.rs")
}

/// Graph construction validated all endpoints. The same closure rule used for
/// a complete witness can test membership in a singleton by comparing IDs.
pub(super) fn check_singleton(
    positions: &Positions,
    edges: &[StateEdge],
    query: ReachQuery,
) -> Result<(), CertificateError> {
    positions
        .get(query.start)
        .ok_or(GraphError::UnknownOp(query.start))?;
    if query.kinds == super::EdgeKinds::NONE {
        return Ok(());
    }
    let first_open = |direction: super::Direction| {
        if query.kinds == super::EdgeKinds::ALL {
            return edges.iter().position(|edge| {
                let (from, to) = direction.endpoints(edge);
                !rule::closed(from == query.start, true, to == query.start)
            });
        }
        edges.iter().position(|edge| {
            let (from, to) = direction.endpoints(edge);
            from == query.start
                && !rule::closed(true, query.kinds.contains(edge.kind), to == query.start)
        })
    };
    let ordinal = match query.direction {
        super::Direction::Forward => first_open(super::Direction::Forward),
        super::Direction::Reverse => first_open(super::Direction::Reverse),
    };
    if let Some(ordinal) = ordinal {
        return Err(CertificateError::OpenEdge(ordinal as u32));
    }
    Ok(())
}
