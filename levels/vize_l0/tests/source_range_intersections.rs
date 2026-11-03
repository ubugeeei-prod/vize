//! Half-open source ranges intersect only when they share an actual byte.

use serde::Deserialize;
use vize_l0::source_range::SourceRange;

#[derive(Deserialize)]
struct Case {
    left: [u32; 2],
    right: [u32; 2],
    intersects: bool,
}

#[test]
fn source_range_intersection_corpus_preserves_half_open_boundaries() {
    let cases: [Case; 16] = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compat/source-range-intersections-8639/cases.json"
    ))
    .unwrap();

    for case in cases {
        let left = SourceRange::new(case.left[0], case.left[1]);
        let right = SourceRange::new(case.right[0], case.right[1]);
        assert_eq!(left.intersects(&right), case.intersects, "{left} / {right}");
        assert_eq!(right.intersects(&left), case.intersects, "{right} / {left}");
        assert_eq!(left.intersection(&right).is_some(), case.intersects);
        assert_eq!(right.intersection(&left).is_some(), case.intersects);
    }
}

#[test]
fn every_small_source_range_intersection_matches_shared_byte_membership() {
    for left_start in 0..=6 {
        for left_end in 0..=6 {
            for right_start in 0..=6 {
                for right_end in 0..=6 {
                    let left = SourceRange::new(left_start, left_end);
                    let right = SourceRange::new(right_start, right_end);
                    // The standard half-open iterator supplies an independent
                    // byte-set oracle, including empty and reversed ranges.
                    let shared_byte =
                        (left_start..left_end).any(|byte| (right_start..right_end).contains(&byte));
                    assert_eq!(left.intersects(&right), shared_byte, "{left} / {right}");
                    assert_eq!(left.intersection(&right).is_some(), shared_byte);
                }
            }
        }
    }
}

#[test]
fn point_and_saturated_source_ranges_do_not_gain_byte_membership() {
    let enclosing = SourceRange::new(10, 20);
    for empty in [
        SourceRange::point(15),
        SourceRange::new(16, 14),
        SourceRange::new(1, 2).offset(-2),
        SourceRange::new(u32::MAX - 1, u32::MAX).offset(1),
    ] {
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        let start_membership = empty.contains(empty.start);
        assert!(!start_membership);
        assert!(!empty.intersects(&enclosing));
        assert!(!enclosing.intersects(&empty));
        assert_eq!(empty.intersection(&enclosing), None);
    }
}
