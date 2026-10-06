//! Complete public #7915 originals, configured continuation units and fixpoints.
use serde::Deserialize;
use vize_glyph::{FormatOptions, format_sfc};
use vize_l0::String;

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
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/css-continuation-indent-7915/cases.json"
    )).expect("whole public regression corpus");
    assert_eq!(corpus.issue, 7915);
    assert_eq!(corpus.cases.len(), 12);
    for case in corpus.cases {
        let mut current = case.input;
        for pass in 1..=3 {
            let actual = format_sfc(&current, &case.options).expect("actual SFC formatter");
            assert_eq!(actual.code, case.expected, "{} whole pass {pass}", case.id);
            current = actual.code;
        }
    }
}
