//! Arena allocator: the per-compile allocation handle.
//!
//! One `Allocator` value carries the whole compile — the template tree, the
//! retained oxc ASTs P1-5 parses, and every arena-resident string — in a
//! single pool with a single reset point.
//!
//! Davinci P1-1 introduced this handle as a *pair* of pools (a `bumpalo::Bump`
//! for the template containers plus an [`oxc_allocator::Allocator`] for
//! retained ASTs) because oxc's arena containers reject `Drop` payloads at
//! compile time and the template nodes still owned heap strings. P1-10's
//! string diet removed the last of those owners, so the bump pool is gone and
//! `Allocator` is a transparent wrapper over the oxc arena: one physical pool,
//! and the const assertion inside [`crate::Box`] / [`crate::Vec`] now stands
//! guard over every arena-resident type in the compiler.
//!
//! # Lifetime contract (Davinci P1-11)
//!
//! The handle is **per compile**, and under the batch pool ([`crate::pool`])
//! the same handle serves file after file with [`Allocator::reset`] in
//! between. One rule makes that safe:
//!
//! > Every value that outlives a compile is in its **owned form** before the
//! > reset. Arena-backed values (`&'a str`, `Box<'a, _>`, `Vec<'a, _>`, the
//! > retained oxc ASTs, every `RootNode<'a>`) are per-compile ephemera:
//! > caches, stage artifacts and diagnostics never hold an `&'a` reference and
//! > never pin an arena.
//!
//! The rule is enforced first by the borrow checker — `reset` takes `&mut
//! self` and the pool guard owns its handle, so an arena-backed value that
//! survived the reset is a compile error, not a runtime bug. The
//! `#[cfg(debug_assertions)]` generation counter below covers the seams
//! borrow checking cannot see (owned values parked with
//! [`Allocator::alloc_owned`], pointer round-trips, a pool that forgot to
//! reset): [`ArenaStamp`] carries the arena identity plus its generation, and
//! reading through a stale stamp panics loudly instead of returning recycled
//! bytes.

mod generation;

use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;

use oxc_allocator::GetAllocator;

pub use generation::ArenaStamp;

/// Arena allocator for Vize.
///
/// A per-compile allocation handle. Template structures, retained oxc ASTs and
/// arena strings all allocate from the same pool, so a `&'a Allocator` hands
/// out `&'a` references that share one lifetime, and [`Allocator::reset`]
/// clears all of them at once — nothing allocated here survives a compile
/// boundary.
///
/// # Example
///
/// ```
/// use vize_carton::{Allocator, Box, Vec};
///
/// let allocator = Allocator::default();
/// let allocator = &allocator;
///
/// let s = allocator.alloc_str("hello");
/// assert_eq!(s, "hello");
///
/// let boxed = Box::new_in(42, &allocator);
/// assert_eq!(*boxed, 42);
///
/// let mut vec = Vec::new_in(&allocator);
/// vec.push(1);
/// assert_eq!(vec.len(), 1);
/// ```
#[derive(Default)]
pub struct Allocator {
    oxc: oxc_allocator::Allocator,
    /// Owned values parked for the compile's duration by
    /// [`Allocator::alloc_owned`]. Held as boxes rather than arena bytes so
    /// their destructors run at [`Allocator::reset`] — see that method.
    parked: RefCell<std::vec::Vec<std::boxed::Box<dyn Any + Send>>>,
    #[cfg(debug_assertions)]
    identity: generation::ArenaIdentity,
}

impl Allocator {
    /// Creates a new allocator. The pool starts empty and reserves lazily.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new allocator with the specified capacity reserved.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            oxc: oxc_allocator::Allocator::with_capacity(capacity),
            ..Self::default()
        }
    }

    /// Allocates a string slice in the arena.
    #[inline]
    pub fn alloc_str(&self, s: &str) -> &str {
        self.oxc.alloc_str(s)
    }

    /// Parks an owned value for the rest of the compile and borrows it at the
    /// arena's lifetime.
    ///
    /// The only callers are the compile entry points that receive an owned
    /// cross-compile summary (`Croquis`) by value and must hand the transform
    /// a reference at the arena's lifetime — a local cannot produce one,
    /// because `'a` is chosen by the caller and outlives the whole body.
    ///
    /// P1-10 implemented this by parking a `ManuallyDrop<T>` in the arena
    /// itself, reproducing what `bumpalo::Bump::alloc` did before: the arena
    /// never runs destructors, so the summary's own heap allocations were
    /// never released. Per compile that is a leak, and P1-11's batch pool
    /// compiles thousands of files per process, so the leak became the peak-RSS
    /// story. The value therefore lives in a `Box` owned by the allocator now:
    /// the borrow still ends at the arena's lifetime, and [`Allocator::reset`]
    /// (and dropping the allocator) runs the real destructor.
    ///
    /// Arena-resident *nodes* stay `Drop`-free — the container const
    /// assertions in [`crate::Box`] / [`crate::Vec`] keep them that way — and
    /// this entry point does not widen for them: the `Any` bound implies
    /// `'static`, so a parked value cannot hold an arena reference in the
    /// first place. It takes owned values built outside the arena, nothing
    /// else.
    #[inline]
    pub fn alloc_owned<T: Any + Send>(&self, value: T) -> &T {
        let ptr: *const T = {
            let mut parked = self.parked.borrow_mut();
            parked.push(std::boxed::Box::new(value));
            // Borrow the value *after* the box is in place: taking the pointer
            // before the push and then moving the box would invalidate it
            // (moving a `Box` retags its pointee), which Miri rejects.
            parked
                .last()
                .expect("the value was just pushed")
                .downcast_ref::<T>()
                .expect("the value was just pushed as a `T`")
        };
        // SAFETY: `ptr` points into a heap box owned by `self.parked`, so the
        // value stays at that address for as long as the box lives. Growing
        // the vector relocates the `Box` handles with an untyped copy, which
        // leaves the values they point to untouched. The boxes are dropped
        // only by `reset` and by `Drop for Allocator`, both of which need
        // `&mut self` / ownership — excluded for the whole lifetime of the
        // returned `&self`-tied borrow.
        unsafe { &*ptr }
    }

    /// Returns the arena oxc parses retained ASTs into.
    ///
    /// Identical to the pool everything else uses; the accessor is kept
    /// because `oxc_parser` takes `&oxc_allocator::Allocator` by name.
    #[inline]
    pub fn as_oxc(&self) -> &oxc_allocator::Allocator {
        &self.oxc
    }

    /// Resets the allocator, freeing all allocated memory.
    ///
    /// This allows reusing the allocator for a new compilation without
    /// deallocating the underlying memory — the batch pool ([`crate::pool`])
    /// resets between files instead of building a new arena per file.
    ///
    /// Everything allocated since the last reset is invalidated: arena bytes
    /// are recycled (oxc keeps the current chunk and returns the rest to the
    /// global allocator) and every value parked by [`Allocator::alloc_owned`]
    /// is dropped. Taking `&mut self` is what makes that sound — no
    /// arena-backed borrow can be alive here — and in debug builds the arena's
    /// generation advances, so a stamp taken before this call
    /// ([`Allocator::stamp`]) panics when validated afterwards.
    #[inline]
    pub fn reset(&mut self) {
        self.parked.get_mut().clear();
        self.oxc.reset();
        #[cfg(debug_assertions)]
        self.identity.advance();
    }

    /// Returns the number of bytes currently allocated.
    ///
    /// Arena bytes only: values parked by [`Allocator::alloc_owned`] own heap
    /// allocations outside the arena and are not counted.
    #[inline]
    pub fn allocated_bytes(&self) -> usize {
        self.oxc.used_bytes()
    }

    /// Stamps the arena's current identity and generation.
    ///
    /// The stamp is the escape check for values that leave the borrow
    /// checker's sight (a pointer round-trip, an owned park, a handle stored
    /// in a side table): validate it with [`Allocator::assert_stamp_current`]
    /// before reading, and a value that outlived a [`Allocator::reset`] panics
    /// instead of returning another file's bytes. Carries no data in release
    /// builds, where validation is a no-op.
    #[inline]
    pub fn stamp(&self) -> ArenaStamp {
        #[cfg(debug_assertions)]
        {
            self.identity.stamp()
        }
        #[cfg(not(debug_assertions))]
        {
            ArenaStamp::RELEASE
        }
    }

    /// Panics when `stamp` did not come from this arena's current generation.
    ///
    /// No-op in release builds. See [`Allocator::stamp`].
    #[inline]
    pub fn assert_stamp_current(&self, stamp: ArenaStamp) {
        #[cfg(debug_assertions)]
        self.identity.assert_current(stamp);
        #[cfg(not(debug_assertions))]
        let _ = stamp;
    }
}

impl Deref for Allocator {
    type Target = oxc_allocator::Allocator;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.oxc
    }
}

impl AsRef<oxc_allocator::Allocator> for Allocator {
    #[inline]
    fn as_ref(&self) -> &oxc_allocator::Allocator {
        &self.oxc
    }
}

/// Lets `&Allocator` stand in wherever oxc's arena containers want an arena,
/// so `Box::new_in(value, &allocator)` / `Vec::new_in(&allocator)` accept the
/// Vize handle directly.
impl<'a> GetAllocator<'a> for &'a Allocator {
    #[inline]
    fn allocator(&self) -> &'a oxc_allocator::Allocator {
        &self.oxc
    }
}

#[cfg(test)]
mod tests;
