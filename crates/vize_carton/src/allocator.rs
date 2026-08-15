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

use std::ops::Deref;

use oxc_allocator::GetAllocator;

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
}

impl Allocator {
    /// Creates a new allocator. The pool starts empty and reserves lazily.
    #[inline]
    pub fn new() -> Self {
        Self {
            oxc: oxc_allocator::Allocator::new(),
        }
    }

    /// Creates a new allocator with the specified capacity reserved.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            oxc: oxc_allocator::Allocator::with_capacity(capacity),
        }
    }

    /// Allocates a string slice in the arena.
    #[inline]
    pub fn alloc_str(&self, s: &str) -> &str {
        self.oxc.alloc_str(s)
    }

    /// Parks an owned value in the arena so it can be borrowed for the whole
    /// compile.
    ///
    /// The arena never runs destructors, so `value`'s own heap allocations are
    /// not released when the arena is — exactly the contract `bumpalo`'s
    /// `alloc` had before P1-10 collapsed the pools. This is deliberately
    /// narrow: the only callers are the compile entry points that receive an
    /// owned cross-compile summary (`Croquis`) by value and must hand the
    /// transform a reference at the arena's lifetime. Arena-resident *nodes*
    /// stay `Drop`-free, and the container const assertions in
    /// [`crate::Box`] / [`crate::Vec`] keep them that way — this escape hatch
    /// does not widen for them. Decoupling the summary's lifetime from the
    /// arena is P1-11's lifetime-contract work.
    #[inline]
    pub fn alloc_owned<T>(&self, value: T) -> &mut T {
        let parked = self.oxc.alloc(std::mem::ManuallyDrop::new(value));
        &mut *parked
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
    /// deallocating the underlying memory.
    #[inline]
    pub fn reset(&mut self) {
        self.oxc.reset();
    }

    /// Returns the number of bytes currently allocated.
    #[inline]
    pub fn allocated_bytes(&self) -> usize {
        self.oxc.used_bytes()
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
mod tests {
    use super::Allocator;
    use crate::{Box, Vec};

    #[test]
    fn test_allocator_new() {
        let allocator = Allocator::new();
        assert_eq!(allocator.allocated_bytes(), 0);
    }

    #[test]
    fn test_allocator_default() {
        let allocator = Allocator::default();
        assert_eq!(allocator.allocated_bytes(), 0);
    }

    #[test]
    fn test_alloc_str() {
        let allocator = Allocator::new();
        let s = allocator.alloc_str("hello world");
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_oxc_pool_allocates_and_counts() {
        let allocator = Allocator::new();
        let s = allocator.as_oxc().alloc_str("retained");
        assert_eq!(s, "retained");
        assert!(allocator.allocated_bytes() > 0);
    }

    #[test]
    fn test_containers_and_strings_share_the_handle_lifetime() {
        let allocator = Allocator::new();
        let allocator = &allocator;
        let from_str = allocator.alloc_str("string side");
        let from_oxc = allocator.as_oxc().alloc_str("retained side");
        let boxed = Box::new_in(7_u32, &allocator);
        // All three references are alive together under the same borrow of
        // `allocator` — the single-lifetime contract P1-5 builds on.
        assert_eq!(
            (from_str, from_oxc, *boxed),
            ("string side", "retained side", 7)
        );
    }

    #[test]
    fn test_reset_clears_the_pool() {
        let mut allocator = Allocator::new();
        {
            let alloc = &allocator;
            let _ = alloc.alloc_str("hello");
            let mut v: Vec<'_, u32> = Vec::new_in(&alloc);
            v.push(1);
        }
        assert!(allocator.allocated_bytes() > 0);
        allocator.reset();
        assert_eq!(allocator.allocated_bytes(), 0);
    }
}
