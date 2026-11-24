//! Check the actual interval constructor, visitor and unique selection against
//! the authored half-open spans. Distinct rows can have identical endpoints.
#![allow(
    unexpected_cfgs,
    reason = "Kani selects its symbolic harness with cfg(kani)"
)]

use super::Intervals;
use crate::query::PositionQueryError;
use vize_l0::Span;

fn check_queries<const N: usize>(bounds: [[u32; 2]; N], offset: u32, selected: u8) {
    let spans = bounds.map(|[start, end]| Span::new(start, end));
    let index = Intervals::from_rows(&spans, |span| *span);
    check_index(&spans, &index, offset, selected);
}

fn check_index(spans: &[Span], index: &Intervals<usize>, offset: u32, selected: u8) {
    let mut expect = 0u8;
    for (i, span) in spans.iter().enumerate() {
        if span.start <= offset && offset < span.end {
            expect |= 1 << i;
        }
    }
    let mut seen = 0u8;
    index
        .visit(offset, &mut |span, &i| {
            assert_eq!(spans[i], span);
            assert_eq!(seen & (1 << i), 0);
            seen |= 1 << i;
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(seen, expect);

    let matches = expect & selected;
    let expected = match matches.count_ones() {
        0 => Ok(None),
        1 => Ok(Some(matches.trailing_zeros() as usize)),
        _ => Err(PositionQueryError::AmbiguousSite),
    };
    let actual = index.unique(offset, |_, &i| (selected & (1 << i) != 0).then_some(i));
    assert_eq!(actual, expected);
}

#[test]
fn query_selections_preserve_crossings_and_equal_spans_at_byte_boundaries() {
    let rows = [
        [0, 0],
        [0, 1],
        [1, 3],
        [0, 2],
        [2, 0],
        [2, u32::MAX],
        [u32::MAX - 1, u32::MAX],
    ];
    for first in rows {
        for second in rows {
            for third in rows {
                for offset in [0, 1, 2, 3, u32::MAX - 1, u32::MAX] {
                    for selected in 0..8 {
                        check_queries([first, second, third], offset, selected);
                    }
                }
            }
        }
    }
}

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(4)]
fn position_queries_preserve_original_row_numbers() {
    let offset = kani::any();
    let selected = kani::any();
    // The last row duplicates the first. Removing the middle row leaves a gap
    // in the original row numbers, even though the physical index is dense.
    for first in [[u32::MAX - 4, u32::MAX], [2, 1]] {
        for second in [[0, u32::MAX - 1], [0, 0]] {
            for third in [[u32::MAX - 4, u32::MAX], [u32::MAX, u32::MAX]] {
                check_queries([first, second, third], offset, selected);
            }
        }
    }
}

#[cfg(kani)]
fn check_physical<const N: usize>(spans: [Span; N], offset: u32, selected: u8) {
    use super::Entry;
    use alloc::vec::Vec;

    let entries: [Entry<usize>; N] = core::array::from_fn(|i| Entry::new(spans[i], i));
    let index = Intervals::sorted(Vec::from(entries));
    check_index(&spans, &index, offset, selected);
}

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(4)]
fn position_queries_for_unordered_intervals() {
    let a = Span::new(kani::any(), kani::any());
    let b = Span::new(kani::any(), kani::any());
    let c = Span::new(kani::any(), kani::any());
    let offset = kani::any();
    let selected = kani::any();
    // This is the exact filter used by from_rows. Discarding an empty or
    // reversed span cannot remove a match at any query offset.
    assert_eq!(!a.is_empty(), a.start < a.end);
    assert!(!a.is_empty() || !(a.start <= offset && offset < a.end));
    check_physical([], offset, selected);
    check_physical([a], offset, selected);
    check_physical([a, b], offset, selected);
    check_physical([a, b, c], offset, selected);
}
