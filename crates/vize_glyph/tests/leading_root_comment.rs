//! Complete public original and independently authored first-root attachment laws.

use serde::Deserialize;
use vize_glyph::{EndOfLine, FormatOptions, format_sfc};
use vize_l0::String;

#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
    expected: String,
    eol: Option<String>,
}

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[test]
fn first_root_comments_keep_whole_attachment_and_standalone_references() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/leading-root-comment-8117/cases.json"
    ))
    .unwrap();
    assert_eq!(corpus.cases.len(), 12);
    for case in corpus.cases {
        let options = FormatOptions {
            end_of_line: if case.eol.as_deref() == Some("auto") {
                EndOfLine::Auto
            } else {
                EndOfLine::Lf
            },
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

#[test]
fn complete_pinned_habitica_task_keeps_next_line_comment_and_fixed_point() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/leading-root-comment-8117/habitica/task.vue.txt"
    );
    assert_eq!(source.len(), 34_878);
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    assert!(first.code.starts_with(concat!(
        "<!-- eslint-enable max-len -->\n",
        "<!-- eslint-disable-next-line vue/component-tags-order -->\n",
        "<script>\n",
    )));
    assert_eq!(first.changed, first.code.as_str() != source);
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();
    assert_eq!(first.code, second.code, "complete second output");
    assert_eq!(second.code, third.code, "complete third output");
    assert!(!second.changed);
    assert!(!third.changed);
}
