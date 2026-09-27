//! The one arena every level's artifacts share.
#![expect(clippy::todo, reason = "skeleton: #6834")]

use core::marker::PhantomData;

/// A bump arena. Artifacts of every level borrow from one arena per session,
/// so dropping a level's artifact is dropping the arena, not walking a tree.
#[derive(Debug)]
pub struct Arena {
    _not_sync: PhantomData<*mut ()>,
}

impl Arena {
    /// An empty arena.
    #[must_use]
    pub fn new() -> Self {
        todo!()
    }

    /// Move `value` into the arena.
    pub fn alloc<T>(&self, value: T) -> &mut T {
        let _ = value;
        todo!()
    }

    /// Copy `text` into the arena.
    pub fn alloc_str(&self, text: &str) -> &str {
        let _ = text;
        todo!()
    }

    /// Bytes allocated so far.
    #[must_use]
    pub fn allocated_bytes(&self) -> usize {
        todo!()
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
