//! Direct engine regressions, without Vize's catch_unwind or pre-parse guards.
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::traits::Parse;
use lightningcss::values::color::{HSL, SRGB};
use lightningcss::values::percentage::Percentage;

#[test]
fn percentage_math_returns_an_error_without_panicking() {
  for source in [
    "abs(-50%)",
    "sign(-50%)",
    "asin(5)",
    "calc(5)",
    "calc(1 + 1)",
    "sqrt(2)",
    "sign(5)",
    "mod(-50%, 2)",
    "min(-50%, 2)",
  ] {
    assert!(Percentage::parse_string(source).is_err(), "{source}");
  }
  for (source, expected) in [("50%", 0.5), ("calc(50% + 10%)", 0.6), ("min(10%, 20%)", 0.1)] {
    assert_eq!(Percentage::parse_string(source).unwrap().0, expected, "{source}");
  }
}

#[test]
fn actual_f32_hue_census_retains_healthy_bits_and_bounds_fallbacks() {
  let mut healthy = 0;
  let mut finite_fallback = 0;
  let mut non_finite = 0;
  let mut nan = 0;
  let mut positive_infinity = 0;
  let mut negative_infinity = 0;
  for high_bits in 0..=u16::MAX {
    let hue = f32::from_bits(u32::from(high_bits) << 16);
    if hue.is_nan() {
      nan += 1;
    } else if hue == f32::INFINITY {
      positive_infinity += 1;
    } else if hue == f32::NEG_INFINITY {
      negative_infinity += 1;
    }
    // resolve_missing replaces NaN with zero before the original computation.
    let resolved = if hue.is_nan() { 0.0 } else { hue };
    let old_hue = (resolved - 360.0 * (resolved / 360.0).floor()) / 360.0;
    let expected_hue = if (0.0..=1.0).contains(&old_hue) {
      healthy += 1;
      old_hue
    } else if resolved.is_finite() {
      finite_fallback += 1;
      resolved.rem_euclid(360.0) / 360.0
    } else {
      non_finite += 1;
      0.0
    };
    assert!((0.0..=1.0).contains(&expected_hue));
    let (r, g, b) = cssparser_color::hsl_to_rgb(expected_hue, 1.0, 0.5);
    let actual = SRGB::from(HSL {
      h: hue,
      s: 100.0,
      l: 50.0,
      alpha: 1.0,
    });
    assert_eq!(
      [actual.r.to_bits(), actual.g.to_bits(), actual.b.to_bits()],
      [r.to_bits(), g.to_bits(), b.to_bits()],
      "hue bits {:08x}",
      hue.to_bits()
    );
    assert_eq!(actual.alpha.to_bits(), 1.0_f32.to_bits());
  }
  assert_eq!(healthy + finite_fallback + non_finite, 65_536);
  assert_eq!([healthy, finite_fallback, non_finite], [61_660, 3_874, 2]);
  assert_eq!([nan, positive_infinity, negative_infinity], [254, 1, 1]);
  println!("VIZE_HUE_CENSUS_COMPLETE=65536 healthy={healthy} finite={finite_fallback} nonfinite={non_finite} nan={nan} positive_infinity={positive_infinity} negative_infinity={negative_infinity}");
}

#[test]
fn whole_original_fuzz_artifacts_finish_direct_parse_and_print() {
  for (source, bytes) in [
    (include_str!("vize_fixtures/lch-sign-original.css.txt"), 39),
    (include_str!("vize_fixtures/number-fold-original.css.txt"), 66),
    (include_str!("vize_fixtures/hue-assert-original.css.txt"), 1045),
  ] {
    assert_eq!(source.len(), bytes);
    // Invalid mangled CSS may return an ordinary parser error. Both paths must
    // execute the real engine without a panic-catching wrapper.
    if let Ok(stylesheet) = StyleSheet::parse(source, ParserOptions::default()) {
      stylesheet.to_css(PrinterOptions::default()).unwrap();
    }
  }
}
