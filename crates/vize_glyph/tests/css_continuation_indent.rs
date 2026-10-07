//! Complete public #7915 originals, configured continuation units and fixpoints.
use serde::Deserialize;
use vize_glyph::{FormatOptions, format_sfc};
use vize_l0::String;

#[path = "support/css_multi_value_reference.rs"]
mod reference;

#[derive(Deserialize)]
struct Corpus {
    issue: u32,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    input: String,
    expected: String,
    options: FormatOptions,
}

#[test]
fn css_declaration_continuations_keep_the_configured_unit_on_every_pass() {
    let raw = include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/css-continuation-indent-7915/cases.json"
    );
    let corpus: Corpus = serde_json::from_str(raw).expect("whole public regression corpus");
    let original: serde_json::Value = serde_json::from_str(raw).expect("whole original plan");
    assert_eq!(corpus.issue, 7915);
    assert_eq!(corpus.cases.len(), 12);
    let mut qualified = 0;
    for (index, case) in corpus.cases.into_iter().enumerate() {
        let current_expected = reference::current_reference(
            "css-continuation-indent-7915",
            raw.as_bytes(),
            &original["cases"][index],
            &case.input,
            &case.expected,
        );
        qualified += usize::from(current_expected.is_some());
        let expected = current_expected.as_deref().unwrap_or(&case.expected);
        let mut current = case.input;
        for pass in 1..=3 {
            let actual = format_sfc(&current, &case.options).expect("actual SFC formatter");
            assert_eq!(
                actual.code.as_str(),
                expected,
                "{} whole pass {pass}",
                case.id
            );
            if current_expected.is_some() {
                assert_ne!(actual.code, case.expected, "strict original mismatch");
            }
            current = actual.code;
        }
    }
    assert_eq!(qualified, 11);
}
