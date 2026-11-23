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

pub(super) fn check_rows(
    ops: &[Op],
    edges: &[StateEdge],
    query: ReachQuery,
    steps: &[ReachStep],
) -> Result<(), CertificateError> {
    let positions = Positions::read(ops, edges)?;
    let start = positions
        .get(query.start)
        .ok_or(GraphError::UnknownOp(query.start))?;
    let Some((root, rest)) = steps.split_first() else {
        return Err(CertificateError::Root);
    };
    if root.op != query.start || root.via.is_some() {
        return Err(CertificateError::Root);
    }
    let mut seen = vec![false; ops.len()];
    *seen
        .get_mut(start)
        .ok_or(GraphError::UnknownOp(query.start))? = true;
    for step in rest {
        let position = positions
            .get(step.op)
            .ok_or(GraphError::UnknownOp(step.op))?;
        if *seen.get(position).ok_or(GraphError::UnknownOp(step.op))? {
            return Err(CertificateError::Duplicate(step.op));
        }
        let witness = CertificateError::Witness { op: step.op };
        let ordinal = step.via.ok_or_else(|| witness.clone())?;
        let edge = edges.get(ordinal as usize).ok_or_else(|| witness.clone())?;
        let (from, to) = query.direction.endpoints(edge);
        let prior = positions.endpoint(ordinal, from)?;
        let prior = *seen.get(prior).ok_or(GraphError::UnknownOp(from))?;
        if !rule::justifies(
            prior,
            query.kinds.contains(edge.kind),
            to.index(),
            step.op.index(),
        ) {
            return Err(witness);
        }
        *seen
            .get_mut(position)
            .ok_or(GraphError::UnknownOp(step.op))? = true;
    }
    for (ordinal, edge) in edges.iter().enumerate() {
        let ordinal = ordinal as u32;
        let (from, to) = query.direction.endpoints(edge);
        let parent = *seen
            .get(positions.endpoint(ordinal, from)?)
            .ok_or(GraphError::UnknownOp(from))?;
        let child = *seen
            .get(positions.endpoint(ordinal, to)?)
            .ok_or(GraphError::UnknownOp(to))?;
        if !rule::closed(parent, query.kinds.contains(edge.kind), child) {
            return Err(CertificateError::OpenEdge(ordinal));
        }
    }
    Ok(())
}
