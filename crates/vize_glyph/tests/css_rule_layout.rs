//! Complete original selector lists and rule gaps, with three public fixed points.

use serde::Deserialize;
use std::{fs, path::Path};
use vize_glyph::{FormatOptions, format_sfc};
use vize_l0::String;

#[path = "support/css_multi_value_reference.rs"]
mod reference;

#[derive(Deserialize)]
struct Corpus {
    schema: String,
    version: u8,
    issues: Vec<u32>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
    expected: String,
    #[serde(default)]
    options: FormatOptions,
}

#[test]
fn original_selector_lists_and_adjacent_rules_keep_complete_layout() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/formatter-regressions/css-rule-layout-7926-7966");
    let raw = fs::read(directory.join("cases.json")).expect("whole authored corpus");
    let corpus: Corpus = serde_json::from_slice(&raw).expect("typed authored corpus");
    let original: serde_json::Value = serde_json::from_slice(&raw).expect("whole original plan");
    assert_eq!(corpus.schema, "vize.formatter-css-rule-layout");
    assert_eq!(corpus.version, 1);
    assert_eq!(corpus.issues, [7926, 7966]);
    assert_eq!(corpus.cases.len(), 17);
    let mut qualified = 0;
    for (index, case) in corpus.cases.into_iter().enumerate() {
        let mut source: String = fs::read_to_string(directory.join(case.source.as_str()))
            .expect("whole original source")
            .into();
        let expected = fs::read_to_string(directory.join(case.expected.as_str()))
            .expect("independently authored whole output");
        let current_expected = reference::current_reference(
            "css-rule-layout-7926-7966",
            &raw,
            &original["cases"][index],
            &source,
            &expected,
        );
        qualified += usize::from(current_expected.is_some());
        let accepted = current_expected.as_deref().unwrap_or(&expected);
        for pass in 1..=3 {
            let result = format_sfc(&source, &case.options).expect("public SFC formatter");
            assert_eq!(result.code.as_str(), accepted, "{} pass {pass}", case.id);
            if current_expected.is_some() {
                assert_ne!(result.code.as_str(), expected, "strict original mismatch");
            }
            source = result.code;
        }
    }
    assert_eq!(qualified, 1);
}
