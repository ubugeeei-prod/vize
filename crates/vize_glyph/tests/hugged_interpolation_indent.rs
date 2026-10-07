//! Public #7969 whole inputs, parent anchors, raw bytes and three fixed-point passes.
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
fn sole_hugged_interpolations_share_the_parent_layout_anchor() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/hugged-interpolation-indent-7969/cases.json"
    )).expect("whole public regression corpus");
    assert_eq!(corpus.issue, 7969);
    assert_eq!(corpus.cases.len(), 13);
    for case in corpus.cases {
        let mut current = case.input;
        for pass in 1..=3 {
            let actual = format_sfc(&current, &case.options).expect("actual SFC formatter");
            assert_eq!(actual.code, case.expected, "{} whole pass {pass}", case.id);
            current = actual.code;
        }
    }
}
