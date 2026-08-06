//! The synthesized `update:` emit payload for `defineModel` (#3904): an
//! optional model without a default carries `T | undefined` — its `ModelRef`
//! type, and what vue-tsc's synthesized listener accepts — while required
//! models and models with defaults keep the bare payload.

use crate::virtual_ts::generate_virtual_ts;

fn emits_of(script: &str) -> std::string::String {
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, "<div>{{ model }}</div>");
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let code = output.code.as_str();
    let start = code.find("export type Emits").expect("Emits alias");
    let end = code[start..].find(";\n").map_or(code.len(), |e| start + e);
    code[start..end].into()
}

#[test]
fn an_optional_model_update_payload_carries_undefined() {
    let emits = emits_of("const model = defineModel<string>()\nvoid model;\n");
    assert!(
        emits.contains("\"update:modelValue\": [value: (string) | undefined]"),
        "optional model must accept undefined in its update payload:\n{emits}"
    );
}

#[test]
fn required_and_defaulted_models_keep_the_bare_payload() {
    let required = emits_of("const model = defineModel<string>({ required: true })\nvoid model;\n");
    assert!(
        required.contains("\"update:modelValue\": [value: string]"),
        "required model keeps the bare payload:\n{required}"
    );
    let defaulted =
        emits_of("const model = defineModel<string>({ default: \"x\" })\nvoid model;\n");
    assert!(
        defaulted.contains("\"update:modelValue\": [value: string]"),
        "defaulted model keeps the bare payload:\n{defaulted}"
    );
}
