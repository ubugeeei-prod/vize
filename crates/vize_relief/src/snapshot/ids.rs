use std::fmt;

/// Opaque node identifier within one [`crate::ReliefSnapshot`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReliefSnapshotNodeId(u32);

impl ReliefSnapshotNodeId {
    /// Reconstruct an ID at a cache or interchange boundary.
    ///
    /// [`crate::ReliefSnapshot::node`] returns `None` when the ID does not
    /// belong to that snapshot.
    #[inline]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Stable integer form within one snapshot.
    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[inline]
    pub(crate) fn from_index(index: usize) -> Self {
        Self(u32::try_from(index).expect("Relief snapshot exceeds u32::MAX nodes"))
    }

    #[inline]
    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Debug for ReliefSnapshotNodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ReliefSnapshotNodeId({})", self.0)
    }
}
