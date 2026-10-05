//! #7826: complete authored CSS/SFC grouping, formatting and fixed points.

use serde::Deserialize;
use vize_glyph::{FormatOptions, format_sfc, format_style};
use vize_l0::String;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Corpus {
    schema: String,
    version: u8,
    issue: u32,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    api: String,
    input: String,
    expected: String,
    options: FormatOptions,
}

#[test]
fn original_rule_groups_and_controls_have_complete_stable_output() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("fixtures/style_rule_blank_lines_7826.json"))
            .expect("authored regression corpus");
    assert_eq!(corpus.schema, "vize.formatter-css-grouping-regressions");
    assert_eq!(
        (corpus.version, corpus.issue, corpus.cases.len()),
        (1, 7826, 27)
    );
    for case in corpus.cases {
        let mut input = case.input;
        for pass in 1..=3 {
            let output = match case.api.as_str() {
                "style" => format_style(&input, &case.options).expect("real CSS formatter"),
                "sfc" => {
                    format_sfc(&input, &case.options)
                        .expect("real SFC formatter")
                        .code
                }
                _ => panic!("unregistered API"),
            };
            assert_eq!(output, case.expected, "case {} pass {pass}", case.id);
            input = output;
        }
    }
}
