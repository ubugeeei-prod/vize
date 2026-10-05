use serde::Deserialize;
use vize_glyph::{FormatOptions, format_sfc};
use vize_l0::String;

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
    expected: String,
}

#[test]
fn reported_expression_groups_remain_whole_and_stable_in_complete_sfcs() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/template-parens-7923/cases.json"
    ))
    .unwrap();
    assert_eq!(corpus.cases.len(), 16);
    let options = FormatOptions::default();
    for case in corpus.cases {
        let mut current = case.source;
        for pass in 1..=3 {
            let formatted = format_sfc(current.as_str(), &options).unwrap();
            assert_eq!(formatted.changed, formatted.code != current, "{}", case.id);
            assert_eq!(formatted.code, case.expected, "{}: pass {pass}", case.id);
            current = formatted.code;
        }
    }
}
