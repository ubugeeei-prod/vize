//! Full original source, finite historical refinements and unchanged layout laws.

use serde::Deserialize;
use vize_glyph::{EndOfLine, FormatOptions, format_sfc};
use vize_l0::String;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    source: String,
    expected: String,
    eol: Option<String>,
    sort_blocks: Option<bool>,
    sort_attributes: Option<bool>,
}

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[test]
fn root_comment_corpus_preserves_whole_outputs_and_three_pass_fixed_points() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/root-comment-attachment-7877/cases.json"
    ))
    .unwrap();
    assert_eq!(corpus.cases.len(), 20);
    for case in corpus.cases {
        let options = FormatOptions {
            end_of_line: match case.eol.as_deref() {
                Some("auto") => EndOfLine::Auto,
                Some("crlf") => EndOfLine::Crlf,
                None => EndOfLine::Lf,
                Some(other) => panic!("undeclared line ending: {other}"),
            },
            sort_blocks: case.sort_blocks.unwrap_or(true),
            sort_attributes: case.sort_attributes.unwrap_or(true),
            ..FormatOptions::default()
        };
        let first = format_sfc(&case.source, &options).unwrap();
        assert_eq!(first.code.as_str(), case.expected.as_str(), "{}", case.id);
        assert_eq!(first.changed, first.code != case.source, "{}", case.id);
        let second = format_sfc(&first.code, &options).unwrap();
        let third = format_sfc(&second.code, &options).unwrap();
        assert_eq!(second.code, first.code, "{}: whole second pass", case.id);
        assert_eq!(third.code, second.code, "{}: whole third pass", case.id);
        assert!(!second.changed, "{}: second pass unchanged", case.id);
        assert!(!third.changed, "{}: third pass unchanged", case.id);
    }
}
