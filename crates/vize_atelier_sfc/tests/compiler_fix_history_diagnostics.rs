//! Complete diagnostic result for historical compiler fix #1416.

use serde_json::Value;
use vize_atelier_core::{CodegenOptions, options::CustomElementMatcher};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

mod support;
use support::fix_history_options as fixture_options;

const SOURCE: &str =
    include_str!("fixtures/fix-history/invalid-template-expression-diagnostic.input.txt");
const OPTIONS: &str =
    include_str!("fixtures/fix-history/invalid-template-expression-diagnostic.options.json");
const EXPECTED: &str =
    include_str!("fixtures/fix-history/invalid-template-expression-diagnostic.expected.json");

#[test]
fn invalid_template_expression_keeps_complete_result() {
    let descriptor_options = SfcParseOptions::default();
    let options = SfcCompileOptions::default();
    fixture_options::validate(
        &serde_json::from_str(OPTIONS).expect("valid immutable option reference"),
        &descriptor_options,
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
    )
    .expect("authored options must match their reference");
    let descriptor = parse_sfc(SOURCE, descriptor_options).expect("parse authored SFC");
    assert_eq!(
        serde_json::to_value(compile_sfc(&descriptor, options))
            .expect("serialize complete public Result"),
        serde_json::from_str::<Value>(EXPECTED).expect("valid immutable Result reference"),
        "complete diagnostic Result changed for fix 336622af2"
    );
}
