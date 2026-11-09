//! Arena generation counter: the debug escape check for cross-file survivals
//! (Davinci P1-11).
//!
//! Under the batch pool one `Allocator` serves file after file, so bytes that
//! belonged to file N are handed to file N+1 after a reset. The borrow checker
//! already rejects the safe form of that mistake — `Allocator::reset` takes
//! `&mut self`, so no `&'a` borrow can be alive across it. What it cannot see
//! is a value that left the lifetime system: an owned park, a pointer stored
//! in a side table, a pool that hands out an arena it forgot to reset.
//!
//! [`ArenaIdentity`] closes that gap in debug builds. Every arena gets a
//! process-unique id at construction and a generation that advances on every
//! reset; an [`ArenaStamp`] records both. Validating a stamp taken before a
//! reset panics loudly instead of reading recycled bytes. In release builds
//! the stamp is a zero-sized value and validation compiles away.

#[cfg(debug_assertions)]
use std::cell::Cell;
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicU64, Ordering};

/// Identity plus generation of the arena a value was allocated from.
///
/// Obtained from `Allocator::stamp` and validated with
/// `Allocator::assert_stamp_current`. Empty in release builds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArenaStamp {
    #[cfg(debug_assertions)]
    arena: u64,
    #[cfg(debug_assertions)]
    generation: u64,
}

impl ArenaStamp {
    /// The single stamp value release builds produce: identity and generation
    /// are debug-only, so there is nothing to distinguish.
    #[cfg(not(debug_assertions))]
    pub(super) const RELEASE: Self = Self {};
}

/// Debug-only arena identity: a process-unique id and the current generation.
#[cfg(debug_assertions)]
#[derive(Debug)]
pub(super) struct ArenaIdentity {
    id: u64,
    generation: Cell<u64>,
}

#[cfg(debug_assertions)]
impl Default for ArenaIdentity {
    fn default() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            generation: Cell::new(0),
        }
    }
}

#[cfg(debug_assertions)]
impl ArenaIdentity {
    /// Stamps the current generation.
    pub(super) fn stamp(&self) -> ArenaStamp {
        ArenaStamp {
            arena: self.id,
            generation: self.generation.get(),
        }
    }

    /// Advances the generation; every stamp taken before this call is stale.
    pub(super) fn advance(&mut self) {
        let next = self.generation.get() + 1;
        self.generation.set(next);
    }

    /// Panics unless `stamp` names this arena's current generation.
    pub(super) fn assert_current(&self, stamp: ArenaStamp) {
        let current = self.stamp();
        assert_eq!(
            stamp, current,
            "arena-backed value outlived its compile: stamped arena #{} generation {}, \
             read against arena #{} generation {}. Every value that crosses a compile \
             boundary must be converted to its owned form before `Allocator::reset`.",
            stamp.arena, stamp.generation, current.arena, current.generation
        );
    }
}
