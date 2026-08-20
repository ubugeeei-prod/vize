use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

#[test]
fn v_model_modifiers_are_replayed_as_modifier_props() {
    let script = r#"import Child from "./Child.vue"
let text = "hello"
"#;
    let template = r#"<Child v-model.trim.capitalize="text" v-model:title.lazy="text" />"#;

    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output.code.contains(r#""modelValue": text"#),
        "argument-less v-model should still pass modelValue:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains(r#""modelModifiers": { "trim": true, "capitalize": true }"#),
        "argument-less v-model modifiers should become modelModifiers:\n{}",
        output.code
    );
    assert!(
        output.code.contains(r#""title": text"#),
        "named v-model should still pass the named prop:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains(r#""titleModifiers": { "lazy": true }"#),
        "named v-model modifiers should use the named modifiers prop:\n{}",
        output.code
    );
}
