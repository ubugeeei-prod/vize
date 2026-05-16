//! Arena allocator wrapping bumpalo.

use bumpalo::Bump;
use std::ops::Deref;

/// Arena allocator for Vize.
///
/// This is a thin wrapper around [`bumpalo::Bump`] that provides arena allocation.
/// Memory is allocated in a contiguous block and freed all at once when the allocator
/// is dropped, making it very efficient for AST construction.
///
/// # Example
///
/// ```
/// use vize_carton::Allocator;
///
/// let allocator = Allocator::default();
/// let s = allocator.alloc_str("hello");
/// assert_eq!(s, "hello");
/// ```
#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}

impl Allocator {
    /// Creates a new allocator.
    #[inline]
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Creates a new allocator with the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
        }
    }

    /// Allocates a string slice in the arena.
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

    /// Resets the allocator, freeing all allocated memory.
    ///
    /// This allows reusing the allocator for a new compilation without
    /// deallocating the underlying memory.
    #[inline]
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Returns the number of bytes currently allocated in the arena.
    #[inline]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
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
    fn test_reset() {
        let mut allocator = Allocator::new();
        let _ = allocator.alloc_str("hello");
        assert!(allocator.allocated_bytes() > 0);
        allocator.reset();
        // Note: allocated_bytes may not be 0 after reset due to chunk reuse
    }
}
