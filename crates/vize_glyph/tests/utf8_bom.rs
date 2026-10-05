use serde::Deserialize;
use vize_glyph::{EndOfLine, FormatOptions, format_json, format_jsonc, format_script, format_sfc};
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
    expected: Option<String>,
    eol: Option<String>,
    outcome: Option<String>,
}

#[test]
fn original_bom_corpus_has_complete_output_and_fixed_points() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/utf8-bom-7870/cases.json"
    ))
    .unwrap();
    assert_eq!(corpus.cases.len(), 14);
    for case in corpus.cases {
        let options = FormatOptions {
            end_of_line: match case.eol.as_deref() {
                Some("auto") => EndOfLine::Auto,
                Some("crlf") => EndOfLine::Crlf,
                None => EndOfLine::Lf,
                Some(other) => panic!("unknown EOL: {other}"),
            },
            ..FormatOptions::default()
        };
        let run = |source: &str| match case.api.as_str() {
            "sfc" => format_sfc(source, &options).map(|result| {
                assert_eq!(
                    result.changed,
                    result.code.as_str() != source,
                    "{}",
                    case.id
                );
                result.code
            }),
            "json" => format_json(source, &options),
            "jsonc" => format_jsonc(source, &options),
            "script" => format_script(source, &options),
            other => panic!("unknown API: {other}"),
        };
        if case.outcome.as_deref() == Some("error") {
            assert!(run(&case.source).is_err(), "{}", case.id);
            continue;
        }
        let expected = case.expected.unwrap();
        let first = run(&case.source).unwrap();
        assert_eq!(
            first.as_str(),
            expected.as_str(),
            "{}: original bytes",
            case.id
        );
        let second = run(first.as_str()).unwrap();
        assert_eq!(second, first, "{}: second pass", case.id);
        assert_eq!(
            run(second.as_str()).unwrap(),
            second,
            "{}: third pass",
            case.id
        );
    }
}
