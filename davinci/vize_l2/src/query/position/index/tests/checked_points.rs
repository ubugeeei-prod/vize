//! Check the actual interval constructor, visitor and unique selection against
//! the authored half-open spans. Distinct rows can have identical endpoints.
#![allow(
    unexpected_cfgs,
    reason = "Kani selects its symbolic harness with cfg(kani)"
)]

use super::Intervals;
use crate::query::PositionQueryError;
use vize_l0::Span;

fn check_queries(bounds: [[u32; 2]; 3], offset: u32, selected: u8) {
    let spans = bounds.map(|[start, end]| Span::new(start, end));
    let index = Intervals::from_rows(&spans, |span| *span);
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
#[kani::unwind(8)]
fn position_visits_match_original_rows() {
    check_queries(kani::any(), kani::any(), kani::any());
}
