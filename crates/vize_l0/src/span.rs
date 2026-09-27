//! Byte spans into one source text.
#![expect(clippy::todo, reason = "skeleton: #6834")]

/// A half-open byte range `start..end` into one source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span {
    /// First byte.
    pub start: u32,
    /// One past the last byte.
    pub end: u32,
}

impl Span {
    /// The span `start..end`.
    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Length in bytes.
    #[must_use]
    pub const fn len(self) -> u32 {
        todo!()
    }

    /// True for an empty span.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        todo!()
    }

    /// This span relative to `base`, for span-relative keys.
    #[must_use]
    pub const fn rebase(self, base: u32) -> Self {
        let _ = base;
        todo!()
    }
}
