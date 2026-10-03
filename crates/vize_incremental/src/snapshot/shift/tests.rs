//! The diagnostic original-For mirror moves its region without a head copy.

use super::shifted;
use crate::snapshot::region::RegionLowering;
use vize_l0::{Span, String};
use vize_l2::dump::{Expr, Interpolation, Op, OriginalFor};

fn original_region(start: u32) -> RegionLowering {
    RegionLowering {
        ops: vec![Op::OriginalFor(OriginalFor {
            node: 7,
            span: Span::new(start, start + 12),
            ops: vec![Op::OriginalFor(OriginalFor {
                node: 8,
                span: Span::new(start + 3, start + 11),
                ops: vec![Op::Interpolation(Interpolation {
                    span: Span::new(start + 6, start + 9),
                    expression: Expr::Js {
                        source: String::from("x"),
                        span: Span::new(start + 7, start + 8),
                    },
                })],
            })],
        })],
        surface: Vec::new(),
        semantic: Vec::new(),
    }
}

#[test]
fn original_regions_shift_once_without_changing_diagnostic_node_numbers() {
    let original = original_region(10);
    assert_eq!(shifted(&original, 10, 20), Some(original_region(20)));
    assert_eq!(shifted(&original, 10, 0), Some(original_region(0)));
    assert_eq!(original, original_region(10));
}

#[test]
fn out_of_range_original_region_spans_refuse_adoption() {
    let original = original_region(10);
    assert_eq!(shifted(&original, 10, u32::MAX - 2), None);
    assert_eq!(shifted(&original, 11, 0), None);
}
