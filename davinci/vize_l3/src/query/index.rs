use super::{Direction, GraphError, L3Ir, Op, OpId, Positions, ReachQuery, ReachStep, StateEdge};
use alloc::{borrow::Cow, vec, vec::Vec};

const END: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Links {
    forward: u32,
    reverse: u32,
}

impl Links {
    const EMPTY: Self = Self {
        forward: END,
        reverse: END,
    };
    fn at(self, direction: Direction) -> u32 {
        match direction {
            Direction::Forward => self.forward,
            Direction::Reverse => self.reverse,
        }
    }
}

/// One read of the edge table constructs both forward and reverse adjacency.
/// A borrow prevents mutation or reuse of offsets against a different IR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpGraph<'p> {
    ops: Cow<'p, [Op]>,
    edges: Cow<'p, [StateEdge]>,
    positions: Positions,
    heads: Vec<Links>,
    links: Vec<Links>,
}

impl<'p> OpGraph<'p> {
    pub fn new(ir: &'p L3Ir<'_>) -> Result<Self, GraphError> {
        let positions = Positions::read(&ir.ops, &ir.edges)?;
        let mut heads = vec![Links::EMPTY; ir.ops.len()];
        let mut links = vec![Links::EMPTY; ir.edges.len()];
        // Prepending in reverse ordinal order preserves the original order at
        // each endpoint. Parallel edges and self-loops each keep their slot.
        for (ordinal, (link, edge)) in links.iter_mut().zip(ir.edges.iter()).enumerate().rev() {
            let ordinal = ordinal as u32;
            let from = positions.endpoint(ordinal, edge.from)?;
            let to = positions.endpoint(ordinal, edge.to)?;
            let head = heads
                .get_mut(from)
                .ok_or(GraphError::UnknownOp(edge.from))?;
            link.forward = core::mem::replace(&mut head.forward, ordinal);
            let head = heads.get_mut(to).ok_or(GraphError::UnknownOp(edge.to))?;
            link.reverse = core::mem::replace(&mut head.reverse, ordinal);
        }
        Ok(Self {
            ops: Cow::Borrowed(&ir.ops),
            edges: Cow::Borrowed(&ir.edges),
            positions,
            heads,
            links,
        })
    }

    /// Retain only the queried operations, edges and their index after the
    /// lowering arena is released. All traversal and checking code is shared.
    #[must_use]
    pub fn into_owned(self) -> OpGraph<'static> {
        OpGraph {
            ops: Cow::Owned(self.ops.into_owned()),
            edges: Cow::Owned(self.edges.into_owned()),
            positions: self.positions,
            heads: self.heads,
            links: self.links,
        }
    }

    /// Read the operation from the same original graph as its relationships.
    #[must_use]
    pub fn operation(&self, id: OpId) -> Option<&Op> {
        self.ops.get(self.positions.get(id)?)
    }

    /// Validate against the retained original rows, independently of the index.
    pub fn check(
        &self,
        query: ReachQuery,
        steps: &[ReachStep],
    ) -> Result<(), super::CertificateError> {
        super::check::check_rows(&self.ops, &self.edges, query, steps)
    }

    /// Check a complete trail response against the original rows of this graph.
    pub fn check_two_hop(
        &self,
        query: super::TwoHopQuery,
        rows: &[super::TwoHopMatch],
    ) -> Result<(), super::TrailError> {
        super::trail::check_rows(&self.ops, &self.edges, query, rows)
    }

    fn incident_at(
        &self,
        position: usize,
        query: ReachQuery,
    ) -> impl Iterator<Item = (u32, &StateEdge)> + '_ {
        let mut next = self
            .heads
            .get(position)
            .map_or(END, |link| link.at(query.direction));
        core::iter::from_fn(move || {
            while next != END {
                let ordinal = next;
                let edge = self.edges.get(ordinal as usize)?;
                next = self.links.get(ordinal as usize)?.at(query.direction);
                if query.kinds.contains(edge.kind) {
                    return Some((ordinal, edge));
                }
            }
            None
        })
    }

    pub fn incident(
        &self,
        query: ReachQuery,
    ) -> Result<impl Iterator<Item = (u32, &StateEdge)> + '_, GraphError> {
        let position = self
            .positions
            .get(query.start)
            .ok_or(GraphError::UnknownOp(query.start))?;
        Ok(self.incident_at(position, query))
    }

    /// Emit each reachable operation exactly once in breadth-first order,
    /// including the seed. Other requested passes can consume each operation
    /// in the callback while this walk records its independently checkable proof.
    pub fn reach_with(
        &self,
        query: ReachQuery,
        mut visit: impl FnMut(&Op),
    ) -> Result<Vec<ReachStep>, GraphError> {
        let start = self
            .positions
            .get(query.start)
            .ok_or(GraphError::UnknownOp(query.start))?;
        let mut reached = vec![false; self.ops.len()];
        *reached
            .get_mut(start)
            .ok_or(GraphError::UnknownOp(query.start))? = true;
        visit(
            self.ops
                .get(start)
                .ok_or(GraphError::UnknownOp(query.start))?,
        );
        let mut steps = vec![ReachStep {
            op: query.start,
            via: None,
        }];
        let mut cursor = 0;
        while let Some(step) = steps.get(cursor) {
            let position = self
                .positions
                .get(step.op)
                .ok_or(GraphError::UnknownOp(step.op))?;
            for (ordinal, edge) in self.incident_at(position, query) {
                let (_, to) = query.direction.endpoints(edge);
                let target = self.positions.endpoint(ordinal, to)?;
                let seen = reached.get_mut(target).ok_or(GraphError::UnknownOp(to))?;
                if !core::mem::replace(seen, true) {
                    visit(self.ops.get(target).ok_or(GraphError::UnknownOp(to))?);
                    steps.push(ReachStep {
                        op: to,
                        via: Some(ordinal),
                    });
                }
            }
            cursor += 1;
        }
        Ok(steps)
    }
}
