//! Kani checks the shipped `Span` length and block-relative arithmetic
//! against the contract in `span.rs`: a span is the half-open byte range
//! `[start, end)`, an inverted span is empty, and the block-relative form
//! subtracts the block start and clamps offsets before it to zero.
//! `original_source_span_preserves_the_selected_bytes` already checks
//! `Span::slice`.
//!
//! No harness has a loop, so every offset and block start ranges over the
//! whole `u32` domain and the check is exhaustive.
use super::Span;

fn any_span() -> Span {
    Span::new(kani::any(), kani::any())
}

#[kani::proof]
fn span_length_counts_the_offsets_it_covers() {
    let span = any_span();
    let offset: u32 = kani::any();
    let inside = span.start <= offset && offset < span.end;
    assert_eq!(span.is_empty(), span.len() == 0);
    // A nonempty span ends exactly `len` bytes after it starts.
    assert!(span.is_empty() || span.start.checked_add(span.len()) == Some(span.end));
    assert!(!inside || offset - span.start < span.len());
    kani::cover!(span.start > span.end && span.len() == 0, "an inverted span");
    kani::cover!(span.len() == u32::MAX, "the whole offset domain");
}

#[kani::proof]
fn span_block_relative_form_ignores_where_the_block_starts() {
    let span = any_span();
    let block_start: u32 = kani::any();
    let relative = span.to_block_relative(block_start);
    // Rebasing never grows a span, and keeps it whole inside the block.
    assert!(relative.len() <= span.len());
    if block_start <= span.start.min(span.end) {
        assert_eq!(relative.len(), span.len());
        assert_eq!(relative.start + block_start, span.start);
        assert_eq!(relative.end + block_start, span.end);
    }
    if block_start > span.start {
        assert_eq!(relative.start, 0);
    }
    // Moving the span and its block together leaves the relative form.
    let shift: u32 = kani::any();
    if let (Some(start), Some(end), Some(block)) = (
        span.start.checked_add(shift),
        span.end.checked_add(shift),
        block_start.checked_add(shift),
    ) {
        assert_eq!(Span::new(start, end).to_block_relative(block), relative);
    }
    kani::cover!(
        block_start > span.start && block_start < span.end && relative.len() > 0,
        "a block starting inside the span"
    );
    kani::cover!(
        shift > 0 && block_start <= span.start && !span.is_empty(),
        "a span moved with its block"
    );
}
