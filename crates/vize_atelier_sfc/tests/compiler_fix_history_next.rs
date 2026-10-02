//! Complete public Results for five historical SFC compiler fixes.
//!
//! The references are copied byte-for-byte from the measured SFC capture
//! archive. They observe the selected legacy compiler route;
//! they do not certify a native level compiler.

use serde_json::Value;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, compile_sfc,
    compile_sfc_for_adapter, parse_sfc,
};

mod support;
use support::fix_history_options as fixture_options;

const IMPORT_SOURCE: &str =
    include_str!("fixtures/fix-history/imported-component-before-prop.input.txt");
const IMPORT_OPTIONS: &str =
    include_str!("fixtures/fix-history/imported-component-before-prop.options.json");
const IMPORT_EXPECTED: &str =
    include_str!("fixtures/fix-history/imported-component-before-prop.expected.json");
const IMPORT_OBSERVED: &str =
    include_str!("fixtures/fix-history/imported-component-before-prop.observation.json");
const CSS_SOURCE: &str = include_str!("fixtures/fix-history/scoped-css-v-bind.input.txt");
const CSS_OPTIONS: &str = include_str!("fixtures/fix-history/scoped-css-v-bind.options.json");
const CSS_EXPECTED: &str = include_str!("fixtures/fix-history/scoped-css-v-bind.expected.json");
const CSS_OBSERVED: &str = include_str!("fixtures/fix-history/scoped-css-v-bind.observation.json");
const LOOP_SOURCE: &str = include_str!("fixtures/fix-history/dynamic-loop-ref-for.input.txt");
const LOOP_OPTIONS: &str = include_str!("fixtures/fix-history/dynamic-loop-ref-for.options.json");
const LOOP_EXPECTED: &str = include_str!("fixtures/fix-history/dynamic-loop-ref-for.expected.json");
const LOOP_OBSERVED: &str =
    include_str!("fixtures/fix-history/dynamic-loop-ref-for.observation.json");
const UNREF_SOURCE: &str = include_str!("fixtures/fix-history/computed-component-unref.input.txt");
const UNREF_OPTIONS: &str =
    include_str!("fixtures/fix-history/computed-component-unref.options.json");
const UNREF_EXPECTED: &str =
    include_str!("fixtures/fix-history/computed-component-unref.expected.json");
const UNREF_OBSERVED: &str =
    include_str!("fixtures/fix-history/computed-component-unref.observation.json");
const DIRECTIVE_SOURCE: &str =
    include_str!("fixtures/fix-history/custom-directive-child-patch.input.txt");
const DIRECTIVE_OPTIONS: &str =
    include_str!("fixtures/fix-history/custom-directive-child-patch.options.json");
const DIRECTIVE_EXPECTED: &str =
    include_str!("fixtures/fix-history/custom-directive-child-patch.expected.json");
const DIRECTIVE_OBSERVED: &str =
    include_str!("fixtures/fix-history/custom-directive-child-patch.observation.json");

fn parse_archive(expected: &str, observed: &str) -> Result<(Value, Value), serde_json::Error> {
    let expected: Value = serde_json::from_str(expected)?;
    let observed: Value = serde_json::from_str(observed)?;
    Ok((expected, observed))
}

#[test]
fn imported_component_precedes_same_name_prop_in_complete_result() {
    let descriptor_options = SfcParseOptions::default();
    let options = SfcCompileOptions {
        script: ScriptCompileOptions {
            is_ts: true,
            ..ScriptCompileOptions::default()
        },
        ..SfcCompileOptions::default()
    };
    fixture_options::validate(
        &serde_json::from_str(IMPORT_OPTIONS).expect("valid immutable option reference"),
        &descriptor_options,
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
    )
    .expect("authored options must match their reference");
    let descriptor = parse_sfc(IMPORT_SOURCE, descriptor_options).expect("parse authored SFC");
    let actual = compile_sfc_for_adapter(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    );
    let (expected, observed) =
        parse_archive(IMPORT_EXPECTED, IMPORT_OBSERVED).expect("valid immutable archive");
    assert_eq!(observed.get("result"), Some(&expected));
    assert_eq!(
        serde_json::to_value(actual).expect("serialize complete public Result"),
        expected,
        "complete SFC result changed for fix ceb0fda9c"
    );
}

#[test]
fn scoped_css_bind_keeps_complete_result_including_css() {
    let descriptor_options = SfcParseOptions::default();
    let options = SfcCompileOptions {
        scope_id: Some("test".into()),
        script: ScriptCompileOptions {
            id: Some("src/Box.vue".into()),
            ..ScriptCompileOptions::default()
        },
        ..SfcCompileOptions::default()
    };
    fixture_options::validate(
        &serde_json::from_str(CSS_OPTIONS).expect("valid immutable option reference"),
        &descriptor_options,
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
    )
    .expect("authored options must match their reference");
    let descriptor = parse_sfc(CSS_SOURCE, descriptor_options).expect("parse authored SFC");
    let (expected, observed) =
        parse_archive(CSS_EXPECTED, CSS_OBSERVED).expect("valid immutable archive");
    assert_eq!(observed.get("result"), Some(&expected));
    assert_eq!(
        serde_json::to_value(compile_sfc(&descriptor, options))
            .expect("serialize complete public Result"),
        expected,
        "complete SFC result changed for fix e9586daff"
    );
}

#[test]
fn dynamic_loop_ref_keeps_complete_separate_template_result() {
    let descriptor_options = SfcParseOptions {
        filename: "CascaderPanel.vue".into(),
        ..SfcParseOptions::default()
    };
    let options = SfcCompileOptions::default();
    fixture_options::validate(
        &serde_json::from_str(LOOP_OPTIONS).expect("valid immutable option reference"),
        &descriptor_options,
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
    )
    .expect("authored options must match their reference");
    let descriptor = parse_sfc(LOOP_SOURCE, descriptor_options).expect("parse authored SFC");
    let actual = compile_sfc_for_adapter(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcScriptOutputMode::SeparateTemplate,
    );
    let (expected, observed) =
        parse_archive(LOOP_EXPECTED, LOOP_OBSERVED).expect("valid immutable archive");
    assert_eq!(observed.get("result"), Some(&expected));
    assert_eq!(
        serde_json::to_value(actual).expect("serialize complete public Result"),
        expected,
        "complete SFC result changed for fix b01f36701"
    );
}

#[test]
fn computed_component_unref_keeps_complete_result() {
    let descriptor_options = SfcParseOptions::default();
    let options = SfcCompileOptions::default();
    fixture_options::validate(
        &serde_json::from_str(UNREF_OPTIONS).expect("valid immutable option reference"),
        &descriptor_options,
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
    )
    .expect("authored options must match their reference");
    let descriptor = parse_sfc(UNREF_SOURCE, descriptor_options).expect("parse authored SFC");
    let (expected, observed) =
        parse_archive(UNREF_EXPECTED, UNREF_OBSERVED).expect("valid immutable archive");
    assert_eq!(observed.get("result"), Some(&expected));
    assert_eq!(
        serde_json::to_value(compile_sfc(&descriptor, options))
            .expect("serialize complete public Result"),
        expected,
        "complete SFC result changed for fix ff008f61e"
    );
}

#[test]
fn custom_directive_children_keep_complete_patch_result() {
    let descriptor_options = SfcParseOptions::default();
    let options = SfcCompileOptions::default();
    fixture_options::validate(
        &serde_json::from_str(DIRECTIVE_OPTIONS).expect("valid immutable option reference"),
        &descriptor_options,
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
    )
    .expect("authored options must match their reference");
    let descriptor = parse_sfc(DIRECTIVE_SOURCE, descriptor_options).expect("parse authored SFC");
    let (expected, observed) =
        parse_archive(DIRECTIVE_EXPECTED, DIRECTIVE_OBSERVED).expect("valid immutable archive");
    assert_eq!(observed.get("result"), Some(&expected));
    assert_eq!(
        serde_json::to_value(compile_sfc(&descriptor, options))
            .expect("serialize complete public Result"),
        expected,
        "complete SFC result changed for fix 466fa3095"
    );
}
