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

mod support;
use support::fix_history_options as fixture_options;

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
const IMPORT_CURRENT: &str =
    include_str!("fixtures/fix-history/dual-script-import-bindings-10903.expected.json");
const IMPORT_OFFICIAL: &str =
    include_str!("fixtures/fix-history/dual-script-import-bindings-10903.official.json.txt");

#[test]
fn separate_template_render_collision_keeps_complete_module_result() {
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
        &serde_json::from_str(RENDER_OPTIONS).expect("valid immutable option reference"),
        &descriptor_options,
        &options,
        &codegen,
        &custom_elements,
    )
    .expect("authored options must match their reference");
    let descriptor = parse_sfc(RENDER_SOURCE, descriptor_options).expect("parse authored SFC");
    let actual = compile_sfc_for_adapter_with_experimental_options(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        custom_elements,
        codegen,
        SfcScriptOutputMode::SeparateTemplate,
        SfcCompileExperimentalOptions::default(),
    );
    let mut expected =
        serde_json::from_str::<Value>(RENDER_EXPECTED).expect("valid immutable Result reference");
    // Standard mode now accepts authored HTML self-closing tags without a warning.
    // Keep the archived reference exact for every other public result field.
    expected["Ok"]["warnings"] = Value::Array(vec![]);
    let expected = support::closed_setup_reference::advance(
        expected,
        "  setup(__props) {\n",
        "  setup(__props, { expose: __expose }) {\n    __expose();\n",
    );
    assert_eq!(
        serde_json::to_value(actual).expect("serialize complete public Result"),
        expected,
        "complete SFC result changed for fix fe724476c"
    );
}

#[test]
fn dual_script_type_imports_keep_complete_module_result() {
    let descriptor_options = SfcParseOptions::default();
    let options = SfcCompileOptions::default();
    fixture_options::validate(
        &serde_json::from_str(IMPORT_OPTIONS).expect("valid immutable option reference"),
        &descriptor_options,
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
    )
    .expect("authored options must match their reference");
    let descriptor = parse_sfc(IMPORT_SOURCE, descriptor_options).expect("parse authored SFC");
    let archived =
        serde_json::from_str::<Value>(IMPORT_EXPECTED).expect("valid original Result reference");
    let expected =
        serde_json::from_str::<Value>(IMPORT_CURRENT).expect("valid measured successor Result");
    let official =
        serde_json::from_str::<Value>(IMPORT_OFFICIAL).expect("valid measured official results");

    // #10903 corrects only the normal-script unknown import's binding kind.
    // Retain the original full result and admit precisely this one metadata field.
    let mut admitted = archived.clone();
    assert_eq!(
        admitted["Ok"]["bindings"]["bindings"]["runtimeValue"],
        "setup-const"
    );
    admitted["Ok"]["bindings"]["bindings"]["runtimeValue"] = Value::from("setup-maybe-ref");
    assert_eq!(
        expected, admitted,
        "only the reviewed import binding may change"
    );

    // The identical source/options were run with pinned official Vue 3.5.41.
    // Both the authored script option and effective inline adapter agree.
    assert_eq!(official["officialVersion"], "3.5.41");
    assert_eq!(official["dialect"], "Vue 3 standard SFC");
    assert_eq!(official["input"].as_str(), Some(IMPORT_SOURCE));
    assert_eq!(
        official["authoredOptions"],
        serde_json::from_str::<Value>(IMPORT_OPTIONS).expect("valid original options")
    );
    assert_eq!(official["archived"], archived);
    let observations = official["observations"]
        .as_array()
        .expect("two official executions");
    assert_eq!(observations.len(), 2);
    for (observation, inline_template) in observations.iter().zip([false, true]) {
        assert_eq!(observation["parseErrors"], serde_json::json!([]));
        assert_eq!(observation["scriptLang"], "ts");
        assert_eq!(observation["setupLang"], "ts");
        assert_eq!(
            observation["apiOptions"],
            serde_json::json!({
                "id": "",
                "inlineTemplate": inline_template,
                "sourceMap": false,
                "isProd": false,
                "propsDestructure": false,
            })
        );
        assert_eq!(
            expected["Ok"]["bindings"]["bindings"], observation["script"]["bindings"],
            "the complete binding map must match both official executions"
        );
    }
    assert_eq!(
        serde_json::to_value(compile_sfc(&descriptor, options))
            .expect("serialize complete public Result"),
        expected,
        "complete SFC result changed beyond the reviewed #10903 successor for fix 92adfb692"
    );
}
