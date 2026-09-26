use vize_l0::cstr;

use super::super::{Dimension, LengthPercentageAuto};
use super::{corpus, parse_dimension, parse_f32, parse_length_percentage_auto};
use super::{parse_positive_point_width, reference};

#[test]
fn numeric_parser_matches_std_bits_and_error_categories() {
    for value in corpus::values() {
        match (parse_f32(&value), value.parse::<f32>()) {
            (Ok(actual), Ok(expected)) => {
                assert_eq!(actual.to_bits(), expected.to_bits(), "input {value:?}");
            }
            (Err(actual), Err(expected)) => {
                assert_eq!(
                    cstr!("{actual:?}"),
                    cstr!("{expected:?}"),
                    "input {value:?}"
                );
                assert_eq!(cstr!("{actual}"), cstr!("{expected}"), "input {value:?}");
            }
            pair => panic!("numeric parsing disagreed for {value:?}: {pair:?}"),
        }
    }
}

#[test]
fn dimension_and_intrinsic_width_consumers_preserve_std_behavior() {
    for value in corpus::values() {
        for input in [&*value, &*cstr!("{value}%")] {
            assert_eq!(
                dimension_bits(parse_dimension(input)),
                dimension_bits(reference::dimension(input)),
                "dimension {input:?}"
            );
            assert_eq!(
                length_bits(parse_length_percentage_auto(input)),
                length_bits(reference::length(input)),
                "inset {input:?}"
            );
            assert_eq!(
                parse_positive_point_width(input),
                reference::positive_width(input),
                "intrinsic width {input:?}"
            );
        }
    }
}

#[test]
fn input_width_retains_finite_positive_ceil_and_saturation_rules() {
    for rejected in [
        "auto", "AUTO", "0", "-0", "-1", "NaN", "-NaN", "Inf", "1e9999", "50%", " 1",
    ] {
        assert_eq!(
            parse_positive_point_width(rejected),
            None,
            "input {rejected:?}"
        );
    }
    assert_eq!(parse_positive_point_width("0.1"), Some(1));
    assert_eq!(parse_positive_point_width("1.01"), Some(2));
    assert_eq!(parse_positive_point_width("1e-45"), Some(1));
    assert_eq!(parse_positive_point_width("3e38"), Some(usize::MAX));
    assert_eq!(parse_f32("-0").unwrap().to_bits(), (-0.0_f32).to_bits());
    assert!(parse_f32("-NaN").unwrap().is_sign_negative());
}

pub(super) fn dimension_bits(value: Dimension) -> (u8, u32) {
    match value {
        Dimension::Auto => (0, 0),
        Dimension::Points(number) => (1, number.to_bits()),
        Dimension::Percent(number) => (2, number.to_bits()),
    }
}

pub(super) fn length_bits(value: LengthPercentageAuto) -> (u8, u32) {
    match value {
        LengthPercentageAuto::Auto => (0, 0),
        LengthPercentageAuto::Points(number) => (1, number.to_bits()),
        LengthPercentageAuto::Percent(number) => (2, number.to_bits()),
    }
}
