//! Stream the original relationship ordinals of each bounded, edge-distinct trail.
use super::OpGraph;
use crate::{
    op::OpId,
    query::{EdgeKinds, EdgePattern, GraphError, ReachQuery},
};
use core::ops::ControlFlow;
use vize_l0::{FxHashSet, SmallVec};

/// Repeat one directed relationship pattern over an inclusive range of lengths.
/// A range with its minimum above its maximum has no matches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BoundedTrailQuery {
    pub start: OpId,
    pub step: EdgePattern,
    pub min_hops: u32,
    pub max_hops: u32,
}

/// Original edge ordinals in traversal order. Operations can occur repeatedly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrailMatch<'a> {
    pub end: OpId,
    pub edges: &'a [u32],
}

/// Complete means that every matching row reached the visitor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrailVisit<B> {
    Complete,
    Break(B),
    BudgetExhausted,
}

#[derive(Default)]
struct ActiveEdges {
    ordered: SmallVec<[u32; 8]>,
    membership: Option<FxHashSet<u32>>,
}

impl ActiveEdges {
    fn push_new(&mut self, edge: u32) -> bool {
        if self.ordered.len() == 16 && self.membership.is_none() {
            self.membership = Some(self.ordered.iter().copied().collect());
        }
        let fresh = match &mut self.membership {
            Some(set) => set.insert(edge),
            None => !self.ordered.contains(&edge),
        };
        if fresh {
            self.ordered.push(edge);
        }
        fresh
    }

    fn pop(&mut self) {
        if let Some(edge) = self.ordered.pop()
            && let Some(set) = &mut self.membership
        {
            set.remove(&edge);
        }
    }
}

impl OpGraph<'_> {
    /// Emit every matching prefix before its extensions, visiting siblings by
    /// original edge ordinal. The empty trail needs no edge inspection. Each
    /// inspected incident relationship costs one unit, including an ineligible
    /// kind or a relationship already used in the active trail. Work storage
    /// grows with the deepest trail visited and is released when the call ends.
    pub fn visit_bounded_trails<B>(
        &self,
        query: BoundedTrailQuery,
        mut edge_budget: usize,
        mut visit: impl FnMut(TrailMatch<'_>) -> ControlFlow<B>,
    ) -> Result<TrailVisit<B>, GraphError> {
        let incident = |start| {
            self.incident(ReachQuery {
                start,
                direction: query.step.direction,
                kinds: EdgeKinds::ALL,
            })
        };
        let seed = incident(query.start)?;
        let limit = (query.max_hops as usize).min(self.edges.len());
        if query.min_hops as usize > limit {
            return Ok(TrailVisit::Complete);
        }
        if query.min_hops == 0
            && let ControlFlow::Break(value) = visit(TrailMatch {
                end: query.start,
                edges: &[],
            })
        {
            return Ok(TrailVisit::Break(value));
        }
        if limit == 0 || query.step.kinds == EdgeKinds::NONE {
            return Ok(TrailVisit::Complete);
        }
        let mut pending = SmallVec::<[_; 4]>::new();
        pending.push(seed);
        let mut path = ActiveEdges::default();
        while let Some(cursor) = pending.last_mut() {
            let Some((ordinal, edge)) = cursor.next() else {
                pending.pop();
                path.pop();
                continue;
            };
            if edge_budget == 0 {
                return Ok(TrailVisit::BudgetExhausted);
            }
            edge_budget -= 1;
            if !query.step.kinds.contains(edge.kind) || !path.push_new(ordinal) {
                continue;
            }
            let (_, end) = query.step.direction.endpoints(edge);
            let depth = path.ordered.len();
            if depth >= query.min_hops as usize
                && let ControlFlow::Break(value) = visit(TrailMatch {
                    end,
                    edges: &path.ordered,
                })
            {
                return Ok(TrailVisit::Break(value));
            }
            if depth < limit {
                pending.push(incident(end)?);
            } else {
                path.pop();
            }
        }
        Ok(TrailVisit::Complete)
    }
}

mod certificate;
pub use certificate::TrailWitnessError;
