use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;

#[test]
fn generic_props_call_merges_static_and_dynamic_class_bindings() {
    let script = r#"import TeacherCard from "./TeacherCard.vue"
const teacher = { name: "Ada" }
const isLoading = false
"#;
    let template = r#"<TeacherCard
  class="ma-1"
  :teacher="teacher"
  :class="{ 'loading-place-holder': isLoading }"
/>"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output
            .code
            .contains("\"class\": [\"ma-1\", { 'loading-place-holder': isLoading }]"),
        "expected merged class binding in generic props call:\n{}",
        output.code
    );
    assert_eq!(
        output.code.matches("\"class\":").count(),
        1,
        "class should be emitted once in the props object:\n{}",
        output.code
    );
}

#[test]
fn component_prop_check_maps_synthetic_name_to_bound_expression() {
    let script = r#"import Child from "./Child.vue"
const benchmarkMirror = 1
"#;
    let template = r#"<Child :code="String(benchmarkMirror)" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let expression = "String(benchmarkMirror)";
    let source_start = template.find(expression).expect("bound expression present");
    let source_range = source_start..source_start + expression.len();
    let sub_span = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| span.src_range == source_range)
        .expect("component prop check should preserve the exact bound expression range");

    assert!(
        output.code[sub_span.gen_range.clone()].starts_with("__vize_prop_check_"),
        "synthetic check identifier should map to the bound expression: {sub_span:?}"
    );
}
