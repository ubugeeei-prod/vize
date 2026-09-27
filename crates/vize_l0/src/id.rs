//! Dense ids: the keys every side table and cross-level reference uses.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use core::num::NonZeroU32;

/// A level-local node identity. `Option<NodeId>` stays 4 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(NonZeroU32);

const _: () = assert!(size_of::<Option<NodeId>>() == 4);

impl NodeId {
    /// The id for a 0-based index; `None` only for `u32::MAX`.
    #[must_use]
    pub const fn from_index(index: u32) -> Option<Self> {
        let _ = index;
        todo!()
    }

    /// The 0-based index this id denotes.
    #[must_use]
    pub const fn index(self) -> u32 {
        todo!()
    }

    /// The next id in sequence, or `None` at exhaustion.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        todo!()
    }
}

/// The identity of one analysis a pass may invalidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnalysisId(u8);

impl AnalysisId {
    /// The id numbered `index` (below [`crate::pass::MAX_ANALYSES`]).
    #[must_use]
    pub const fn new(index: u8) -> Self {
        let _ = index;
        todo!()
    }

    /// The bit this analysis occupies in a preserved set.
    #[must_use]
    pub const fn index(self) -> u8 {
        todo!()
    }
}
