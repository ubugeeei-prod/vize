//! #7968: complete multi-value continuation layout and fixed points.

use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use serde::Deserialize;
use vize_glyph::{FormatOptions, format_sfc, format_style};
use vize_l0::String;

#[derive(Deserialize)]
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

fn css_body<'a>(source: &'a str, api: &str) -> &'a str {
    if api == "style" {
        return source;
    }
    let (_, style) = source.split_once("<style").expect("actual original style");
    let (_, body) = style.split_once('>').expect("style opening delimiter");
    body.split_once("</style>")
        .expect("style closing delimiter")
        .0
}

fn semantic_print(source: &str) -> std::string::String {
    StyleSheet::parse(source, ParserOptions::default())
        .expect("valid original CSS")
        .to_css(PrinterOptions::default())
        .expect("CSS semantic observation")
        .code
}

#[test]
fn multi_value_declarations_keep_semantics_and_complete_stable_layout() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "fixtures/style_multi_value_declarations_7968.json"
    ))
    .expect("independently authored corpus");
    assert_eq!(corpus.schema, "vize.formatter-css-multi-value-regressions");
    assert_eq!(
        (corpus.version, corpus.issue, corpus.cases.len()),
        (1, 7968, 27)
    );
    for case in corpus.cases {
        assert_eq!(
            semantic_print(css_body(&case.input, &case.api)),
            semantic_print(css_body(&case.expected, &case.api)),
            "case {} original and authored reference CSS semantics",
            case.id
        );
        let mut input = case.input;
        for pass in 1..=3 {
            let output = match case.api.as_str() {
                "style" => format_style(&input, &case.options).expect("real CSS formatter"),
                "sfc" => {
                    format_sfc(&input, &case.options)
                        .expect("real SFC formatter")
                        .code
                }
                _ => panic!("unregistered public API"),
            };
            assert_eq!(output, case.expected, "case {} pass {pass}", case.id);
            input = output;
        }
    }
}
