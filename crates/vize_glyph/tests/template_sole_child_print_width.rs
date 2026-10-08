//! Complete independent #7876 child outputs, changed flags and three fixed points.
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
fn original_and_independent_children_keep_complete_fixed_points() {
    let bytes = include_bytes!(
        "../../../tests/_fixtures/differential/formatter-regressions/sole-child-width-7876/references.json"
    );
    assert_eq!(
        digest(bytes).as_str(),
        "ebe294c117dc2e5e055575a110f68e2b5195beed5f2328f6693b3650efb27fe4"
    );
    let corpus: Corpus = serde_json::from_slice(bytes).expect("whole independent reference corpus");
    assert_eq!(corpus.cases.len(), 16);
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
                "{} pass {pass}",
                case.id
            );
            assert_eq!(actual.code, case.expected, "{} whole pass {pass}", case.id);
            current = actual.code;
        }
    }
}
