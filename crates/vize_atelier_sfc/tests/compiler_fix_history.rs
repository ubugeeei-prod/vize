//! Complete module references for two historical SFC compiler fixes.
//!
//! These are ordinary product-output regressions, not proof of a native
//! level compiler or of complete fix-history coverage. Expected payloads
//! are immutable measured references; never regenerate them on failure.

use serde_json::Value;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    compile_sfc, compile_sfc_for_adapter_with_experimental_options, parse_sfc,
};

#[path = "support/fix_history_options.rs"]
mod fixture_options;

const RENDER_SOURCE: &str =
    include_str!("fixtures/fix-history/separate-template-authored-render-binding.input.txt");
const RENDER_OPTIONS: &str =
    include_str!("fixtures/fix-history/separate-template-authored-render-binding.options.json");
const RENDER_EXPECTED: &str =
    include_str!("fixtures/fix-history/separate-template-authored-render-binding.expected.json");
const IMPORT_SOURCE: &str =
    include_str!("fixtures/fix-history/dual-script-inline-type-only-imports.input.txt");
const IMPORT_OPTIONS: &str =
    include_str!("fixtures/fix-history/dual-script-inline-type-only-imports.options.json");
const IMPORT_EXPECTED: &str =
    include_str!("fixtures/fix-history/dual-script-inline-type-only-imports.expected.json");

#[test]
fn separate_template_render_collision_keeps_complete_module_result()
-> Result<(), Box<dyn std::error::Error>> {
    let descriptor_options = SfcParseOptions::default();
    let options = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: "Foo.vue".into(),
            ..SfcParseOptions::default()
        },
        ..SfcCompileOptions::default()
    };
    let codegen = CodegenOptions::default();
    let custom_elements = CustomElementMatcher::default();
    fixture_options::validate(
        &serde_json::from_str(RENDER_OPTIONS)?,
        &descriptor_options,
        &options,
        &codegen,
        &custom_elements,
    )?;
    let descriptor = parse_sfc(RENDER_SOURCE, descriptor_options)?;
    let actual = compile_sfc_for_adapter_with_experimental_options(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        custom_elements,
        codegen,
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    );
    assert_eq!(
        serde_json::to_value(actual)?,
        serde_json::from_str::<Value>(RENDER_EXPECTED)?,
        "complete SFC result changed for fix fe724476c"
    );
    Ok(())
}

#[test]
fn dual_script_type_imports_keep_complete_module_result() -> Result<(), Box<dyn std::error::Error>>
{
    let descriptor_options = SfcParseOptions::default();
    let options = SfcCompileOptions::default();
    fixture_options::validate(
        &serde_json::from_str(IMPORT_OPTIONS)?,
        &descriptor_options,
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
    )?;
    let descriptor = parse_sfc(IMPORT_SOURCE, descriptor_options)?;
    assert_eq!(
        serde_json::to_value(compile_sfc(&descriptor, options))?,
        serde_json::from_str::<Value>(IMPORT_EXPECTED)?,
        "complete SFC result changed for fix 92adfb692"
    );
    Ok(())
}
