//! Receiver admission comes from normalized semantic facts, before emission.

use crate::virtual_ts::{
    ProjectionFeatures, VirtualTsOptions, VirtualTsOutput, generate_virtual_ts_with_offsets,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn output(template: &str, script: Option<&str>, options: &VirtualTsOptions) -> VirtualTsOutput {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    if let Some(script) = script {
        analyzer.analyze_script_setup(script);
    }
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    generate_virtual_ts_with_offsets(&summary, script, Some(&root), 0, 1000, options)
}

fn assert_owned_read(output: &VirtualTsOutput, template: &str, receiver: &str) {
    let authored = 1000 + template.find("$ghost").unwrap();
    let expected_source = authored..authored + "$ghost".len();
    let property = vize_carton::cstr!("{receiver}.$ghost");
    assert_eq!(
        output.code.matches(property.as_str()).count(),
        1,
        "{}",
        output.code
    );
    let key = output.code.find(property.as_str()).unwrap() + receiver.len() + 1;
    assert!(output.mapping.spans().iter().any(|row| {
        row.gen_range == (key..key + "$ghost".len()) && row.src_range == expected_source
    }));
    assert!(
        output
            .mapping
            .spans()
            .iter()
            .enumerate()
            .any(|(index, row)| {
                row.gen_range == (key - receiver.len() - 1..key)
                    && row.src_range == expected_source
                    && output.mapping.meta(index).features == ProjectionFeatures::DIAGNOSTICS
            })
    );
}

#[test]
fn ordinary_source_retains_the_existing_receiver_and_code_shape() {
    let template = "<div>{{ $ghost.value }}</div>";
    let result = output(template, Some("const rows = [1];"), &Default::default());
    assert_eq!(
        result
            .code
            .matches("const __vize_template_instance")
            .count(),
        0
    );
    assert_owned_read(&result, template, "__ctx");
}

#[test]
fn loop_and_plain_or_escaped_callback_binders_cannot_capture_the_receiver() {
    for template in [
        "<div v-for=\"__ctx in rows\">{{ $ghost.value }}</div>",
        "<div>{{ [1].map((__ctx: any) => $ghost.value) }}</div>",
        r"<div>{{ [1].map((__\u0063tx: any) => $ghost.value) }}</div>",
    ] {
        let result = output(template, Some("const rows = [1];"), &Default::default());
        assert_eq!(
            result
                .code
                .matches("const __vize_template_instance = __ctx;")
                .count(),
            1,
            "{}",
            result.code
        );
        assert_owned_read(&result, template, "__vize_template_instance");
    }
}

#[test]
fn a_non_dollar_unresolved_read_is_reserved_even_without_script_undefined_facts() {
    let template =
        "<div v-for=\"__ctx in [1]\">{{ $ghost.value }}{{ __vize_template_instance }}</div>";
    let result = output(template, None, &Default::default());
    assert_eq!(
        result
            .code
            .matches("const __vize_template_instance_ = __ctx;")
            .count(),
        1,
        "{}",
        result.code
    );
    assert_eq!(
        result
            .code
            .matches("void (__vize_template_instance);")
            .count(),
        1
    );
    assert_owned_read(&result, template, "__vize_template_instance_");
}

#[test]
fn unresolved_type_queries_are_not_bound_by_the_new_alias() {
    let template = "<div>{{ ((__ctx: any) => { type T = typeof __vize_template_instance; return $ghost.value; })(0) }}</div>";
    let result = output(template, Some("const n = 1;"), &Default::default());
    assert_eq!(
        result
            .code
            .matches("const __vize_template_instance_ = __ctx;")
            .count(),
        1,
        "{}",
        result.code
    );
    assert_eq!(
        result
            .code
            .matches("typeof __vize_template_instance;")
            .count(),
        1
    );
    assert_owned_read(&result, template, "__vize_template_instance_");
}

#[test]
fn an_unrelated_function_name_or_outer_script_binding_is_not_a_capture() {
    for (template, script) in [
        (
            "<div>{{ ({ f: function __ctx() { return 1; }, value: $ghost.value }) }}</div>",
            "const n = 1;",
        ),
        (
            "<div>{{ $ghost.value }}</div>",
            "const __ctx = { different: 1 };",
        ),
    ] {
        let result = output(template, Some(script), &Default::default());
        assert_eq!(
            result
                .code
                .matches("const __vize_template_instance")
                .count(),
            0
        );
        assert_owned_read(&result, template, "__ctx");
    }
}
