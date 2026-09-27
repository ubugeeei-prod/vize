//! Complete diagnostic result for historical compiler fix #1416.

use serde_json::Value;
use vize_atelier_core::{CodegenOptions, options::CustomElementMatcher};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[path = "support/fix_history_options.rs"]
mod fixture_options;

const SOURCE: &str =
    include_str!("fixtures/fix-history/invalid-template-expression-diagnostic.input.txt");
const OPTIONS: &str =
    include_str!("fixtures/fix-history/invalid-template-expression-diagnostic.options.json");
const EXPECTED: &str =
    include_str!("fixtures/fix-history/invalid-template-expression-diagnostic.expected.json");

#[test]
fn invalid_template_expression_keeps_complete_result() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_options = SfcParseOptions::default();
    let options = SfcCompileOptions::default();
    fixture_options::validate(
        &serde_json::from_str(OPTIONS)?,
        &descriptor_options,
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
    )?;
    let descriptor = parse_sfc(SOURCE, descriptor_options)?;
    assert_eq!(
        serde_json::to_value(compile_sfc(&descriptor, options))?,
        serde_json::from_str::<Value>(EXPECTED)?,
        "complete diagnostic Result changed for fix 336622af2"
    );
    Ok(())
}
