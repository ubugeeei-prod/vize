//! The Davinci span type: two byte offsets into the authored file.
//!
//! `Span` is the S0 source-model coordinate (`davinci-road/architecture.md`):
//! every node records where it came from as a half-open byte range
//! `[start, end)` into the one authored source string, and nothing else.
//! Text is recovered on demand with [`Span::slice`]; line/column exist only
//! at diagnostic-rendering time, derived from offsets (see
//! [`crate::line_index::LineIndex`]). This is the replacement for the owned
//! `SourceLocation { start: Position, end: Position, source: String }`
//! carried by every relief node today.
//!
//! Landed unused by production code (Davinci P0-9); the migration map lives
//! in `davinci-road/plan/sourcelocation-inventory.md`.
//!
//! # Offset contract
//!
//! Offsets are **UTF-8 byte offsets** measured from the start of the file the
//! span was produced against. Producers (parsers) only ever cut on character
//! boundaries, so well-formed spans always land on `char` boundaries of that
//! source. [`Span::slice`] is nevertheless total: it never panics on
//! malformed input, it clamps and snaps as documented there.

/// A half-open byte range `[start, end)` into an authored source string.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Span {
    /// Start byte offset (inclusive).
    pub start: u32,
    /// End byte offset (exclusive).
    pub end: u32,
}

/// `Span` is passed and stored by value everywhere; keep it two words at most.
const _: () = assert!(core::mem::size_of::<Span>() == 8);

impl Span {
    /// Create a new span. No ordering is enforced; consumers treat an
    /// inverted span (`start > end`) as empty.
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Byte length of the span. Inverted spans (`start > end`) saturate to 0.
    #[inline]
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Whether the span covers no bytes (`start >= end`).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Slice `source` to the text this span covers.
    ///
    /// Never panics. The exact semantics, in order:
    ///
    /// 1. `end` is clamped to `source.len()`, then `start` is clamped to the
    ///    clamped `end` (an inverted or out-of-range span yields `""`).
    /// 2. Each clamped offset is snapped **down** to the nearest `char`
    ///    boundary of `source`. Snapping is monotone, so the result is always
    ///    a valid, possibly empty, subslice.
    ///
    /// For the documented contract — byte offsets produced on character
    /// boundaries of `source` — both steps are identity and this is exactly
    /// `&source[start..end]`. Callers slicing a *different* string than the
    /// one the span was measured against get deterministic garbage, not a
    /// panic; that mismatch is a caller bug the type cannot detect.
    #[inline]
    pub fn slice<'a>(&self, source: &'a str) -> &'a str {
        let end = (self.end as usize).min(source.len());
        let start = (self.start as usize).min(end);
        &source[floor_char_boundary(source, start)..floor_char_boundary(source, end)]
    }

    /// Rebase this span from file-absolute offsets to block-relative offsets
    /// by subtracting `block_start` from both ends.
    ///
    /// Saturating: offsets before `block_start` clamp to 0, so a span that
    /// starts before the block degenerates toward an empty span at the block
    /// start instead of wrapping.
    #[inline]
    pub const fn to_block_relative(self, block_start: u32) -> Span {
        Span {
            start: self.start.saturating_sub(block_start),
            end: self.end.saturating_sub(block_start),
        }
    }

    /// Hash the **block-relative** form of this span, per the rustc
    /// relative-span import: equal content at different absolute positions
    /// (e.g. the same template block shifted by an edit above it) hashes
    /// equally when each side passes its own block start.
    ///
    /// Exactly equivalent to `self.to_block_relative(block_start)` hashed
    /// with the derived [`core::hash::Hash`] impl.
    #[inline]
    pub fn hash_relative<H: core::hash::Hasher>(&self, hasher: &mut H, block_start: u32) {
        core::hash::Hash::hash(&self.to_block_relative(block_start), hasher);
    }
}

/// Largest index `<= index` that is a `char` boundary of `source`.
///
/// `index` must be `<= source.len()` (callers clamp first). A UTF-8 sequence
/// is at most 4 bytes, so at most 3 probes run.
#[inline]
fn floor_char_boundary(source: &str, mut index: usize) -> usize {
    while !source.is_char_boundary(index) {
        index -= 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use core::hash::{Hash, Hasher};
    use std::hash::DefaultHasher;

    use super::Span;

    fn hash_of(value: impl Hash) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    fn relative_hash_of(span: Span, block_start: u32) -> u64 {
        let mut hasher = DefaultHasher::new();
        span.hash_relative(&mut hasher, block_start);
        hasher.finish()
    }

    #[test]
    fn size_is_eight_bytes() {
        assert_eq!(core::mem::size_of::<Span>(), 8);
        assert_eq!(core::mem::align_of::<Span>(), 4);
    }

    #[test]
    fn new_stores_offsets_verbatim() {
        let span = Span::new(3, 7);
        assert_eq!(span.start, 3);
        assert_eq!(span.end, 7);
        let inverted = Span::new(7, 3);
        assert_eq!(inverted.start, 7);
        assert_eq!(inverted.end, 3);
    }

    #[test]
    fn len_counts_bytes() {
        assert_eq!(Span::new(3, 7).len(), 4);
        assert_eq!(Span::new(0, 0).len(), 0);
        assert_eq!(Span::new(5, 5).len(), 0);
        assert_eq!(Span::new(0, u32::MAX).len(), u32::MAX);
    }

    #[test]
    fn len_saturates_on_inverted_spans() {
        assert_eq!(Span::new(7, 3).len(), 0);
        assert_eq!(Span::new(u32::MAX, 0).len(), 0);
    }

    #[test]
    fn is_empty_is_start_at_or_past_end() {
        assert!(Span::new(0, 0).is_empty());
        assert!(Span::new(5, 5).is_empty());
        assert!(Span::new(7, 3).is_empty());
        assert!(!Span::new(3, 7).is_empty());
        assert!(!Span::new(0, 1).is_empty());
    }

    #[test]
    fn slice_returns_covered_text() {
        let source = "let x = 1;";
        assert_eq!(Span::new(0, 3).slice(source), "let");
        assert_eq!(Span::new(4, 5).slice(source), "x");
        assert_eq!(Span::new(0, 10).slice(source), "let x = 1;");
    }

    #[test]
    fn slice_of_empty_span_is_empty() {
        assert_eq!(Span::new(4, 4).slice("let x = 1;"), "");
        assert_eq!(Span::new(0, 0).slice(""), "");
    }

    #[test]
    fn slice_at_end_of_source() {
        let source = "abc";
        assert_eq!(Span::new(3, 3).slice(source), "");
        assert_eq!(Span::new(2, 3).slice(source), "c");
    }

    #[test]
    fn slice_clamps_end_past_source() {
        let source = "abc";
        assert_eq!(Span::new(1, 100).slice(source), "bc");
        assert_eq!(Span::new(0, u32::MAX).slice(source), "abc");
    }

    #[test]
    fn slice_clamps_start_past_source_to_empty() {
        let source = "abc";
        assert_eq!(Span::new(50, 100).slice(source), "");
        assert_eq!(Span::new(3, 100).slice(source), "");
    }

    #[test]
    fn slice_of_inverted_span_is_empty() {
        assert_eq!(Span::new(7, 3).slice("let x = 1;"), "");
        assert_eq!(Span::new(u32::MAX, 0).slice("abc"), "");
    }

    #[test]
    fn slice_on_char_boundaries_of_multibyte_source() {
        // "aéb": a=0..1, é=1..3, b=3..4
        let source = "a\u{e9}b";
        assert_eq!(Span::new(1, 3).slice(source), "\u{e9}");
        assert_eq!(Span::new(0, 4).slice(source), "a\u{e9}b");
        assert_eq!(Span::new(3, 4).slice(source), "b");
    }

    #[test]
    fn slice_snaps_mid_char_offsets_down() {
        // "aéb": offset 2 is inside the two-byte é (1..3).
        let source = "a\u{e9}b";
        // Mid-char start snaps down to 1: the é is included.
        assert_eq!(Span::new(2, 4).slice(source), "\u{e9}b");
        // Mid-char end snaps down to 1: nothing of the é remains.
        assert_eq!(Span::new(1, 2).slice(source), "");
        // Both ends inside the same char collapse to empty.
        assert_eq!(Span::new(2, 2).slice(source), "");
        // Four-byte scalar: "🦀" is 0..4; every interior offset snaps to 0.
        let crab = "\u{1f980}";
        assert_eq!(Span::new(1, 3).slice(crab), "");
        assert_eq!(Span::new(0, 3).slice(crab), "");
        assert_eq!(Span::new(0, 4).slice(crab), "\u{1f980}");
    }

    #[test]
    fn to_block_relative_subtracts_block_start() {
        assert_eq!(Span::new(10, 14).to_block_relative(10), Span::new(0, 4));
        assert_eq!(Span::new(10, 14).to_block_relative(3), Span::new(7, 11));
        assert_eq!(Span::new(10, 14).to_block_relative(0), Span::new(10, 14));
    }

    #[test]
    fn to_block_relative_saturates_before_block_start() {
        // Block start inside the span: start clamps to 0, end still rebases.
        assert_eq!(Span::new(10, 14).to_block_relative(12), Span::new(0, 2));
        // Block start past the span: both clamp to 0.
        assert_eq!(Span::new(10, 14).to_block_relative(20), Span::new(0, 0));
        assert_eq!(Span::new(0, 0).to_block_relative(u32::MAX), Span::new(0, 0));
    }

    #[test]
    fn hash_relative_equal_content_at_different_absolute_positions() {
        // The same 4-byte content in blocks starting at 10 and 50.
        assert_eq!(
            relative_hash_of(Span::new(10, 14), 10),
            relative_hash_of(Span::new(50, 54), 50),
        );
        // Offset within the block is preserved: 12..14 in block 10 matches
        // 52..54 in block 50.
        assert_eq!(
            relative_hash_of(Span::new(12, 14), 10),
            relative_hash_of(Span::new(52, 54), 50),
        );
    }

    #[test]
    fn hash_relative_distinguishes_block_offset_and_length() {
        // Same absolute span, different block starts: different relative form.
        assert_ne!(
            relative_hash_of(Span::new(12, 14), 10),
            relative_hash_of(Span::new(12, 14), 12),
        );
        // Same start, different lengths.
        assert_ne!(
            relative_hash_of(Span::new(10, 14), 10),
            relative_hash_of(Span::new(10, 15), 10),
        );
    }

    #[test]
    fn hash_relative_matches_derived_hash_of_rebased_span() {
        let span = Span::new(37, 41);
        assert_eq!(
            relative_hash_of(span, 30),
            hash_of(span.to_block_relative(30)),
        );
        // With block_start 0 it is exactly the derived hash of the span.
        assert_eq!(relative_hash_of(span, 0), hash_of(span));
    }
}
