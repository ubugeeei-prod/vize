//! Arena allocator: the unified per-compile allocation handle.
//!
//! One `Allocator` value carries the compile's two memory pools under a single
//! lifetime and a single reset point (Davinci P1-1, transitional per charter
//! #26 until P1-10):
//!
//! - a [`bumpalo::Bump`] pool backing the template-side arena containers
//!   (`vize_carton::Box` / `vize_carton::Vec`), which still run element
//!   destructors — required while nodes own heap strings;
//! - an [`oxc_allocator::Allocator`] pool for retained oxc ASTs (expressions
//!   parsed once, P1-5). oxc's arena types reject `Drop` payloads at compile
//!   time, which is why the container aliases cannot move there until the
//!   string diet (P1-3/P1-10) lands.
//!
//! At the pinned oxc revision, `oxc_allocator` is backed by oxc's own `Arena`
//! (not bumpalo), so the two pools cannot physically alias; the handle is the
//! unification. P1-10 deletes the bump pool and flips the aliases, collapsing
//! to one physical pool.

use bumpalo::Bump;
use std::ops::Deref;

/// Arena allocator for Vize.
///
/// A per-compile allocation handle: template structures allocate from the
/// bump pool (via [`Deref`] to [`Bump`] and the `vize_carton::Box`/`Vec`
/// containers), retained oxc ASTs from the oxc pool (via [`Allocator::as_oxc`]).
/// Both pools share the handle's lifetime — a `&'a Allocator` hands out `&'a`
/// references from either pool — and [`Allocator::reset`] clears both, so
/// nothing allocated here survives a compile boundary.
///
/// # Example
///
/// ```
/// use vize_carton::Allocator;
///
/// let allocator = Allocator::default();
/// let s = allocator.alloc_str("hello");
/// let t = allocator.as_oxc().alloc_str("world");
/// assert_eq!(s, "hello");
/// assert_eq!(t, "world");
/// ```
#[derive(Default)]
pub struct Allocator {
    bump: Bump,
    oxc: oxc_allocator::Allocator,
}

impl Allocator {
    /// Creates a new allocator. Both pools start empty and reserve lazily.
    #[inline]
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            oxc: oxc_allocator::Allocator::new(),
        }
    }

    /// Creates a new allocator with the specified capacity.
    ///
    /// The capacity is reserved in the bump pool (today's dominant load: the
    /// template tree). The oxc pool reserves lazily on first retained-AST
    /// allocation.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
            oxc: oxc_allocator::Allocator::new(),
        }
    }

    /// Allocates a string slice in the bump pool.
    #[inline]
    pub fn alloc_str(&self, s: &str) -> &str {
        self.bump.alloc_str(s)
    }

    /// Returns a reference to the underlying bumpalo allocator.
    ///
    /// This is useful for interoperability with code that expects a raw `Bump`.
    #[inline]
    pub fn as_bump(&self) -> &Bump {
        &self.bump
    }

    /// Returns the oxc arena for retained ASTs parsed with `oxc_parser`.
    ///
    /// References allocated here live exactly as long as references from the
    /// bump pool: until this handle is reset or dropped. Values crossing a
    /// compile boundary (caches, folios, summaries) must be converted to
    /// their owned form first — the arena/cache contract.
    #[inline]
    pub fn as_oxc(&self) -> &oxc_allocator::Allocator {
        &self.oxc
    }

    /// Resets the allocator, freeing all allocated memory in both pools.
    ///
    /// This allows reusing the allocator for a new compilation without
    /// deallocating the underlying memory.
    #[inline]
    pub fn reset(&mut self) {
        self.bump.reset();
        self.oxc.reset();
    }

    /// Returns the number of bytes currently allocated across both pools.
    #[inline]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes() + self.oxc.used_bytes()
    }
}

impl Deref for Allocator {
    type Target = Bump;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.bump
    }
}

// Allow using Allocator where Bump is expected via AsRef
impl AsRef<Bump> for Allocator {
    #[inline]
    fn as_ref(&self) -> &Bump {
        &self.bump
    }
}

#[cfg(test)]
mod tests {
    use super::Allocator;

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
    fn test_pools_share_the_handle_lifetime() {
        let allocator = Allocator::new();
        let from_bump = allocator.alloc_str("bump side");
        let from_oxc = allocator.as_oxc().alloc_str("oxc side");
        // Both references are alive together under the same borrow of
        // `allocator` — the single-lifetime contract P1-5 builds on.
        assert_eq!((from_bump, from_oxc), ("bump side", "oxc side"));
    }

    #[test]
    fn test_reset_clears_both_pools() {
        let mut allocator = Allocator::new();
        let _ = allocator.alloc_str("hello");
        let _ = allocator.as_oxc().alloc_str("world");
        assert!(allocator.allocated_bytes() > 0);
        allocator.reset();
        // oxc's used_bytes drops to zero on reset; bumpalo may retain its
        // largest chunk but reports it as free, so only assert the oxc side
        // and that nothing panics.
        assert_eq!(allocator.as_oxc().used_bytes(), 0);
    }
}
