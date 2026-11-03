use super::{Placement, PlacementSet};
use crate::op::OpId;

/// The placement alternatives recorded on one op, plus the committed choice.
///
/// An op without a record has exactly the canonical [`Placement::Inline`]
/// shape. A record therefore always lists `inline` and at least one other
/// alternative, and records appear in program op order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementRecord {
    /// The op whose work may move.
    pub op: OpId,
    /// Every semantics-preserving shape available to the op.
    pub alternatives: PlacementSet,
    /// Head of the effect unit the `group` alternative joins.
    pub leader: Option<OpId>,
    /// The committed shape. Canonical programs choose `inline`.
    pub chosen: Placement,
}

impl PlacementRecord {
    /// A record that still has the canonical `inline` choice.
    #[must_use]
    pub const fn new(op: OpId, alternatives: PlacementSet, leader: Option<OpId>) -> Self {
        Self {
            op,
            alternatives,
            leader,
            chosen: Placement::Inline,
        }
    }

    /// The same record with `chosen` committed.
    #[must_use]
    pub const fn with_chosen(mut self, chosen: Placement) -> Self {
        self.chosen = chosen;
        self
    }
}
