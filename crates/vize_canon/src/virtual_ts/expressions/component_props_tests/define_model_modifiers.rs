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

#[test]
fn generated_model_modifier_keys_map_to_exact_authored_bytes() {
    let script = "import Child from './Child.vue'; let text = '';";
    let template = "<!-- 日本語 -->\r\n<Child v-model.trim.bogus=\"text\" v-model:title.upper.修飾=\"text\" />";
    let offset = 57;
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), offset);
    for modifier in ["trim", "bogus", "upper", "修飾"] {
        let needle = vize_carton::cstr!("\"{modifier}\": true");
        let authored = template
            .find(vize_carton::cstr!(".{modifier}").as_str())
            .unwrap()
            + 1
            + offset as usize;
        let mut mapped = 0;
        for (generated, _) in output.code.match_indices(needle.as_str()) {
            let quoted_end = generated + needle.len() - ": true".len();
            let actual = crate::virtual_ts::mapping::map_generated_range_to_source(
                &output.mappings,
                generated,
                quoted_end,
            );
            assert_eq!(
                actual,
                Some((authored, authored + modifier.len())),
                "{modifier} in {}",
                output.code
            );
            let contents = crate::virtual_ts::mapping::map_generated_range_to_source(
                &output.mappings,
                generated + 1,
                quoted_end - 1,
            );
            assert_eq!(
                contents, actual,
                "quoted and unquoted {modifier} must agree"
            );
            mapped += 1;
        }
        assert_eq!(
            mapped, 2,
            "per-prop and generic prop checks must both own {modifier}"
        );
    }
}
