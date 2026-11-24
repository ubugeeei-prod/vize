//! Verify one claimed trail directly against the retained original relationships.
use super::{BoundedTrailQuery, OpGraph, OpId, SmallVec, TrailMatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrailWitnessError {
    UnknownStart(OpId),
    Length,
    MissingEdge(u32),
    Step(u32),
    RepeatedEdge(u32),
    End { actual: OpId, claimed: OpId },
}

impl OpGraph<'_> {
    /// Verify membership of one answer, including its claimed endpoint. This
    /// reads the original relation rows and checks uniqueness in sorted order.
    pub fn check_bounded_trail(
        &self,
        query: BoundedTrailQuery,
        row: TrailMatch<'_>,
    ) -> Result<(), TrailWitnessError> {
        if self.operation(query.start).is_none() {
            return Err(TrailWitnessError::UnknownStart(query.start));
        }
        if !(query.min_hops as usize..=query.max_hops as usize).contains(&row.edges.len()) {
            return Err(TrailWitnessError::Length);
        }
        let mut at = query.start;
        for &ordinal in row.edges {
            let edge = self
                .edges
                .get(ordinal as usize)
                .ok_or(TrailWitnessError::MissingEdge(ordinal))?;
            let (from, to) = query.step.direction.endpoints(edge);
            if from != at || !query.step.kinds.contains(edge.kind) {
                return Err(TrailWitnessError::Step(ordinal));
            }
            at = to;
        }
        if row.end != at {
            return Err(TrailWitnessError::End {
                actual: at,
                claimed: row.end,
            });
        }
        let mut ordinals = SmallVec::<[u32; 8]>::from_slice(row.edges);
        ordinals.sort_unstable();
        if let Some(&[a, _]) = ordinals.array_windows().find(|[a, b]| a == b) {
            return Err(TrailWitnessError::RepeatedEdge(a));
        }
        Ok(())
    }
}
