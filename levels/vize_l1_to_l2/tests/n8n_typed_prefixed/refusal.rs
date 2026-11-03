//! A strict wrong-language refusal control, separate from admitted TS evidence.
use super::{compile, contract, errors_wire};
use vize_l0::Span;
use vize_l1_to_l2::{EmitError, UnsupportedReason};

pub fn assert_original_js_refusal(name: &str, source: &str) {
    let (_, errors, native) = compile(name, source, false);
    let expected = contract(name);
    // These literal wire goldens are frozen independently of this constructor:
    // The historical 66c2 AFTER archive reused the actual 67e7 baseline binary.
    // Its complete packets are retained as baseline evidence; current strict
    // results are authored, source-derived contracts pending hosted execution.
    assert_eq!(errors_wire(&errors), expected["jsLegacyErrors"]);
    let error = native.expect_err("wrong JS recipe must not earn successful-module credit");
    assert_eq!(format!("{error:#?}"), expected["jsNativeDebug"]);
    let span = &expected["jsNativeSpan"];
    let start = span["start"].as_u64().expect("frozen start") as u32;
    let end = span["end"].as_u64().expect("frozen end") as u32;
    assert_eq!(
        error,
        EmitError::unsupported_at(
            UnsupportedReason::PrefixExpressionRejected,
            Span::new(start, end)
        )
    );
}
