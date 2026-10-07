//! #7928: complete JSON layout bytes captured from the original report and Oxfmt.
use serde::Deserialize;
use vize_glyph::{EndOfLine, FormatOptions, format_json, format_jsonc};
use vize_l0::String;

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    api: String,
    source: String,
    expected: String,
    #[serde(default)]
    options: FormatOptions,
}

#[test]
fn original_json_layout_corpus_matches_public_apis_and_fixed_points() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/json-layout-7928/cases.json"
    ))
    .unwrap();
    for case in corpus.cases {
        let run = |source: &str| match case.api.as_str() {
            "json" => format_json(source, &case.options),
            "jsonc" => format_jsonc(source, &case.options),
            api => panic!("unknown formatter API: {api}"),
        };
        let first = run(&case.source).unwrap();
        assert_eq!(first, case.expected, "{}: complete output", case.id);
        let second = run(&first).unwrap();
        assert_eq!(second, first, "{}: second pass", case.id);
        assert_eq!(run(&second).unwrap(), first, "{}: third pass", case.id);
        if case.api == "json" {
            let original: serde_json::Value = serde_json::from_str(&case.source).unwrap();
            let formatted: serde_json::Value = serde_json::from_str(&first).unwrap();
            assert_eq!(original, formatted, "{}: JSON value", case.id);
        }
    }
}

#[test]
fn blank_lines_follow_the_authored_auto_line_ending() {
    let options = FormatOptions {
        end_of_line: EndOfLine::Auto,
        ..FormatOptions::default()
    };
    for newline in ["\n", "\r\n", "\r"] {
        let source = "{\n\"a\":1,\n\n\"b\":[1,2]\n}".replace('\n', newline);
        let expected = "{\n  \"a\": 1,\n\n  \"b\": [1, 2]\n}\n".replace('\n', newline);
        let first = format_json(&source, &options).unwrap();
        assert_eq!(first, expected);
        assert_eq!(format_json(&first, &options).unwrap(), first);
    }
}
