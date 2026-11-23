use super::{ScopedResolution, SideTable};
use core::ops::{Deref, DerefMut};

/// Expression resolutions and their allocation policy for this file walk.
pub(crate) struct ExpressionTable<'a> {
    entries: SideTable<ScopedResolution<'a>>,
    pub(crate) in_arena: bool,
}

impl ExpressionTable<'_> {
    pub(super) fn new() -> Self {
        Self {
            entries: SideTable::new(),
            in_arena: false,
        }
    }

    pub(crate) fn reserve_if_empty(&mut self, capacity: usize) {
        if capacity > 0 && self.entries.is_empty() {
            self.entries = SideTable::with_capacity(capacity);
        }
    }
}

impl<'a> Deref for ExpressionTable<'a> {
    type Target = SideTable<ScopedResolution<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for ExpressionTable<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}
