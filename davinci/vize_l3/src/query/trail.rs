//! Two directed, typed relationships joined on their middle operation.
//! Original relationship ordinals identify matches, including parallel edges.

use super::{Direction, EdgeKinds, GraphError, L3Ir, OpGraph, OpId, Positions, ReachQuery};
use core::ops::ControlFlow;

/// Direction and eligible kinds of one relationship in a pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgePattern {
    pub direction: Direction,
    pub kinds: EdgeKinds,
}

impl EdgePattern {
    fn at(self, start: OpId) -> ReachQuery {
        ReachQuery {
            start,
            direction: self.direction,
            kinds: self.kinds,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TwoHopQuery {
    pub start: OpId,
    pub first: EdgePattern,
    pub second: EdgePattern,
}

/// A trail may revisit an operation. Its two edges must have distinct identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TwoHopMatch {
    pub first_edge: u32,
    pub second_edge: u32,
    pub middle: OpId,
    pub end: OpId,
}

impl OpGraph<'_> {
    /// Visit ordered relationship pairs without materializing an intermediate
    /// table. A consumer can stop at any row, including the first.
    pub fn visit_two_hop<B>(
        &self,
        query: TwoHopQuery,
        mut visit: impl FnMut(TwoHopMatch) -> ControlFlow<B>,
    ) -> Result<ControlFlow<B>, GraphError> {
        for (first_edge, first) in self.incident(query.first.at(query.start))? {
            let (_, middle) = query.first.direction.endpoints(first);
            for (second_edge, second) in self.incident(query.second.at(middle))? {
                if first_edge != second_edge {
                    let (_, end) = query.second.direction.endpoints(second);
                    let row = TwoHopMatch {
                        first_edge,
                        second_edge,
                        middle,
                        end,
                    };
                    if let ControlFlow::Break(value) = visit(row) {
                        return Ok(ControlFlow::Break(value));
                    }
                }
            }
        }
        Ok(ControlFlow::Continue(()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrailError {
    Graph(GraphError),
    /// A row with these two original edge ordinals was expected here.
    Mismatch {
        at: usize,
        first_edge: u32,
        second_edge: u32,
    },
    /// The claim continues after the last actual match.
    Extra {
        at: usize,
    },
}

/// Independently check a complete ordered response against the original L3
/// rows. This reads the relation directly and validates the entire input graph,
/// including endpoints outside the selected neighbourhood.
pub fn check_two_hop_matches(
    ir: &L3Ir<'_>,
    query: TwoHopQuery,
    rows: &[TwoHopMatch],
) -> Result<(), TrailError> {
    let positions = Positions::read(&ir.ops, &ir.edges).map_err(TrailError::Graph)?;
    positions
        .get(query.start)
        .ok_or(TrailError::Graph(GraphError::UnknownOp(query.start)))?;
    for (i, edge) in ir.edges.iter().enumerate() {
        positions
            .endpoint(i as u32, edge.from)
            .map_err(TrailError::Graph)?;
        positions
            .endpoint(i as u32, edge.to)
            .map_err(TrailError::Graph)?;
    }
    let mut at = 0;
    for (first, left) in ir.edges.iter().enumerate() {
        let (from, middle) = query.first.direction.endpoints(left);
        if from != query.start || !query.first.kinds.contains(left.kind) {
            continue;
        }
        for (second, right) in ir.edges.iter().enumerate() {
            let (through, end) = query.second.direction.endpoints(right);
            if first == second || middle != through || !query.second.kinds.contains(right.kind) {
                continue;
            }
            let first_edge = first as u32;
            let second_edge = second as u32;
            let expected = TwoHopMatch {
                first_edge,
                second_edge,
                middle,
                end,
            };
            if rows.get(at) != Some(&expected) {
                return Err(TrailError::Mismatch {
                    at,
                    first_edge,
                    second_edge,
                });
            }
            at += 1;
        }
    }
    if rows.len() != at {
        return Err(TrailError::Extra { at });
    }
    Ok(())
}
