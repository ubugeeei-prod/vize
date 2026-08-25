//! Validated authored-source frames for Davinci spans.
//!
//! A [`Span`] is only meaningful against the source string it was measured
//! from. SFC lowering parses each block as a slice, but the emitted S0 spans
//! still point into the complete authored file. [`SourceRoot`] and
//! [`SourceBlock`] keep those two facts together without allocating.

use crate::Span;

/// Why a source root/block frame could not be formed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFrameError {
    /// The authored source cannot be addressed by `u32` byte offsets.
    SourceTooLarge,
    /// The requested block range is outside the root source.
    BlockOutOfBounds,
    /// The requested block range does not land on UTF-8 boundaries.
    BlockBoundary,
    /// The block text is not the exact slice at that range in the root.
    BlockNotRootSlice,
}

/// The complete authored source a Davinci artifact is measured against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRoot<'a> {
    source: &'a str,
}

impl<'a> SourceRoot<'a> {
    /// Validate a complete authored source.
    pub fn new(source: &'a str) -> Result<Self, SourceFrameError> {
        if source.len() > u32::MAX as usize {
            return Err(SourceFrameError::SourceTooLarge);
        }
        Ok(Self { source })
    }

    /// The complete authored source.
    #[must_use]
    pub const fn source(self) -> &'a str {
        self.source
    }

    /// A block covering the whole source, with base-zero spans.
    #[must_use]
    pub const fn whole_block(self) -> SourceBlock<'a> {
        SourceBlock {
            root: self.source,
            source: self.source,
            start: 0,
        }
    }

    /// Validate a source block by identity and byte range.
    pub fn block(self, source: &'a str, start: u32) -> Result<SourceBlock<'a>, SourceFrameError> {
        let start = start as usize;
        let end = start
            .checked_add(source.len())
            .ok_or(SourceFrameError::BlockOutOfBounds)?;
        if end > self.source.len() {
            return Err(SourceFrameError::BlockOutOfBounds);
        }
        let Some(root_slice) = self.source.get(start..end) else {
            return Err(SourceFrameError::BlockBoundary);
        };
        if root_slice.as_ptr() != source.as_ptr() {
            return Err(SourceFrameError::BlockNotRootSlice);
        }
        Ok(SourceBlock {
            root: self.source,
            source,
            start: start as u32,
        })
    }

    /// Whether `span` is ordered, inside this root, and on UTF-8 boundaries.
    #[must_use]
    pub fn contains_span(self, span: Span) -> bool {
        contains_span(self.source, span)
    }
}

/// One parsed source block inside a complete authored root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceBlock<'a> {
    root: &'a str,
    source: &'a str,
    start: u32,
}

impl<'a> SourceBlock<'a> {
    /// The complete authored source.
    #[must_use]
    pub const fn root_source(self) -> &'a str {
        self.root
    }

    /// The parsed block source.
    #[must_use]
    pub const fn source(self) -> &'a str {
        self.source
    }

    /// File-absolute byte offset where this block starts.
    #[must_use]
    pub const fn start(self) -> u32 {
        self.start
    }

    /// File-absolute byte offset where this block ends.
    #[must_use]
    pub fn end(self) -> u32 {
        self.start + self.source.len() as u32
    }

    /// The block's file-absolute span.
    #[must_use]
    pub fn span(self) -> Span {
        Span::new(self.start(), self.end())
    }

    /// Whether `span` is ordered, inside the root, and on UTF-8 boundaries.
    #[must_use]
    pub fn contains_root_span(self, span: Span) -> bool {
        contains_span(self.root, span)
    }

    /// Whether `span` is valid and lies inside this block.
    #[must_use]
    pub fn contains_block_span(self, span: Span) -> bool {
        span.start >= self.start && span.end <= self.end() && self.contains_root_span(span)
    }

    /// File-absolute byte offset of `slice` inside this block.
    #[must_use]
    pub fn offset_of(self, slice: &str) -> Option<u32> {
        let base = self.source.as_ptr() as usize;
        let ptr = slice.as_ptr() as usize;
        let end = ptr.checked_add(slice.len())?;
        if ptr < base || end > base + self.source.len() {
            return None;
        }
        let rel = u32::try_from(ptr - base).ok()?;
        Some(self.start + rel)
    }

    /// File-absolute span of `slice` inside this block.
    #[must_use]
    pub fn span_of(self, slice: &str) -> Option<Span> {
        let start = self.offset_of(slice)?;
        let len = u32::try_from(slice.len()).ok()?;
        Some(Span::new(start, start + len))
    }

    /// A zero-width block slice at an authored offset, clamped into the block.
    #[must_use]
    pub fn zero_width_at(self, offset: u32) -> &'a str {
        let start = self.start as usize;
        let end = start + self.source.len();
        let mut at = (offset as usize).clamp(start, end) - start;
        while at > 0 && !self.source.is_char_boundary(at) {
            at -= 1;
        }
        &self.source[at..at]
    }
}

fn contains_span(source: &str, span: Span) -> bool {
    if span.start > span.end {
        return false;
    }
    let start = span.start as usize;
    let end = span.end as usize;
    end <= source.len() && source.is_char_boundary(start) && source.is_char_boundary(end)
}

const _: () = {
    assert!(!core::mem::needs_drop::<SourceRoot<'static>>());
    assert!(!core::mem::needs_drop::<SourceBlock<'static>>());
};

#[cfg(test)]
mod tests {
    use super::{SourceFrameError, SourceRoot};
    use crate::Span;

    #[test]
    fn whole_block_is_base_zero() {
        let source = "abc";
        let root = SourceRoot::new(source).expect("small root");
        let block = root.whole_block();
        assert_eq!(block.root_source(), "abc");
        assert_eq!(block.source(), "abc");
        assert_eq!(block.span(), Span::new(0, 3));
        assert_eq!(block.span_of(&source[1..]), Some(Span::new(1, 3)));
    }

    #[test]
    fn block_requires_the_exact_root_slice() {
        let source = "aa<style>.x{}</style><style>.x{}</style>";
        let first_start = source.find(".x{}").expect("first css");
        let second_start = source.rfind(".x{}").expect("second css");
        let first = &source[first_start..first_start + 4];
        let second = &source[second_start..second_start + 4];
        let root = SourceRoot::new(source).expect("small root");

        let second_block = root
            .block(second, second_start as u32)
            .expect("second block");
        assert_eq!(
            second_block.span(),
            Span::new(second_start as u32, (second_start + second.len()) as u32)
        );
        assert_eq!(
            root.block(first, second_start as u32),
            Err(SourceFrameError::BlockNotRootSlice)
        );
    }

    #[test]
    fn block_rejects_non_boundary_ranges() {
        let source = "a\u{e9}b";
        let block = &source[3..4];
        let root = SourceRoot::new(source).expect("small root");
        assert_eq!(root.block(block, 2), Err(SourceFrameError::BlockBoundary));
    }

    #[test]
    fn span_validation_checks_bounds_order_and_utf8_boundaries() {
        let root = SourceRoot::new("a\u{e9}b").expect("small root");
        assert!(root.contains_span(Span::new(1, 3)));
        assert!(!root.contains_span(Span::new(2, 3)));
        assert!(!root.contains_span(Span::new(3, 99)));
        assert!(!root.contains_span(Span::new(4, 3)));
    }

    #[test]
    fn zero_width_positions_are_clamped_to_the_block() {
        let source = "prefix\n<div>\u{e9}</div>";
        let start = source.find("<div>").expect("block");
        let block_source = &source[start..];
        let root = SourceRoot::new(source).expect("small root");
        let block = root.block(block_source, start as u32).expect("block");

        assert_eq!(block.zero_width_at(0), "");
        assert_eq!(block.zero_width_at((source.len() + 10) as u32), "");
        let mid_char = (source.find('\u{e9}').expect("accent") + 1) as u32;
        assert_eq!(block.zero_width_at(mid_char), "");
    }
}
