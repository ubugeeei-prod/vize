//! Check the shipped span and frame operations against a forward scan of UTF-8
//! leading bytes. Source length is bounded to eight bytes. The public offsets
//! range over the full `u32` domain.
use crate::Span;
use crate::source_frame::{SourceFrameError, SourceRoot};

fn with_source(check: impl FnOnce(&str)) {
    let bytes: [u8; 8] = kani::any();
    let len: usize = kani::any();
    kani::assume(len <= bytes.len());
    if let Ok(source) = core::str::from_utf8(&bytes[..len]) {
        check(source);
    }
}

fn boundary_at_or_before(source: &str, offset: usize) -> usize {
    if offset >= source.len() {
        return source.len();
    }
    let mut last = 0;
    for (index, &byte) in source.as_bytes()[..=offset].iter().enumerate() {
        if byte & 0xc0 != 0x80 {
            last = index;
        }
    }
    last
}

#[kani::proof]
#[kani::unwind(10)]
fn original_source_span_preserves_the_selected_bytes() {
    with_source(|source| {
        let span = Span::new(kani::any(), kani::any());
        let start = span.start as usize;
        let end = span.end as usize;
        let first = boundary_at_or_before(source, start.min(end));
        let last = boundary_at_or_before(source, end);
        let selected = span.slice(source);
        assert_eq!(selected.as_bytes(), &source.as_bytes()[first..last]);
        assert_eq!(selected.as_ptr(), source.as_ptr().wrapping_add(first));

        let exact = start <= end && start == first && end == last;
        assert_eq!(SourceRoot::new(source).unwrap().contains_span(span), exact);
        kani::cover!(
            source.is_empty() && start > end,
            "inverted range in an empty document"
        );
        kani::cover!(
            start < source.len() && start - first == 3 && end > start,
            "inside a four-byte scalar"
        );
        kani::cover!(
            span.end == u32::MAX && !selected.is_empty(),
            "clamped nonempty range"
        );
    });
}

#[kani::proof]
#[kani::unwind(10)]
fn original_source_block_preserves_hole_positions_and_identity() {
    with_source(|source| {
        let start: u8 = kani::any();
        let end: u8 = kani::any();
        let start = usize::from(start);
        let end = usize::from(end);
        let Some(text) = source.get(start..end) else {
            return;
        };
        let root = SourceRoot::new(source).unwrap();
        let block = root.block(text, start as u32).unwrap();
        let offset: u32 = kani::any();
        let clipped = (offset as usize).clamp(start, end);
        let expected = boundary_at_or_before(source, clipped);
        let hole = block.zero_width_at(offset);
        assert!(hole.is_empty());
        assert_eq!(hole.as_ptr(), source.as_ptr().wrapping_add(expected));
        assert!(block.contains_block_span(Span::new(expected as u32, expected as u32)));

        if !text.is_empty() {
            let mut copy = [0; 8];
            copy[..text.len()].copy_from_slice(text.as_bytes());
            let same_spelling = core::str::from_utf8(&copy[..text.len()]).unwrap();
            assert_eq!(
                root.block(same_spelling, start as u32),
                Err(SourceFrameError::BlockNotRootSlice)
            );
        }
        kani::cover!(start > 0 && start == end, "empty block after authored text");
        kani::cover!(
            start > 0 && expected > start && expected < clipped,
            "interior multibyte hole in an offset block"
        );
        kani::cover!(
            start < end && offset == u32::MAX,
            "hole clamped to block end"
        );
    });
}
