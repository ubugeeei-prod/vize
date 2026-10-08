//! Complete independent #7876 closing outputs, changed flags and three fixed points.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use vize_glyph::{FormatOptions, format_sfc};
use vize_l0::String;

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    input: String,
    expected: String,
    input_sha256: String,
    expected_sha256: String,
    options: FormatOptions,
}

fn digest(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        hex.push_str(vize_l0::cstr!("{byte:02x}").as_str());
    }
    hex
}

#[test]
fn independent_closing_grammar_keeps_complete_fixed_points() {
    let bytes = include_bytes!(
        "../../../tests/_fixtures/differential/formatter-regressions/closing-child-width-7876/references.json"
    );
    assert_eq!(
        digest(bytes).as_str(),
        "f57812d0f91255208eca609b3ac7c985b2fbb7c6cfa7f284c4202544b9a23b74"
    );
    let corpus: Corpus = serde_json::from_slice(bytes).expect("whole independent reference corpus");
    assert_eq!(corpus.cases.len(), 13);
    for case in corpus.cases {
        assert_eq!(
            digest(case.input.as_bytes()),
            case.input_sha256,
            "{} input",
            case.id
        );
        assert_eq!(
            digest(case.expected.as_bytes()),
            case.expected_sha256,
            "{} expectation",
            case.id
        );
        let mut current = case.input;
        for pass in 1..=3 {
            let actual = format_sfc(&current, &case.options).expect("public SFC formatter");
            assert_eq!(
                actual.changed,
                current != case.expected,
                "{} pass {pass}; whole actual={:?}; expected={:?}",
                case.id,
                actual.code,
                case.expected
            );
            assert_eq!(actual.code, case.expected, "{} whole pass {pass}", case.id);
            current = actual.code;
        }
    }
}
