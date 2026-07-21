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

#[test]
fn static_attribute_values_are_type_checked_like_dynamic_bindings() {
    let script = r#"import HelloWorld from "./HelloWorld.vue"
"#;
    let template = r#"<HelloWorld msg="You did it!" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert!(
        output.code.contains(
            "type __HelloWorld_0_prop_msg = __VizePropValue<__HelloWorld_Props_0, 'msg'>;"
        ),
        "static prop must declare its child prop type alias:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains(": __HelloWorld_0_prop_msg = \"You did it!\";"),
        "static prop value must be asserted against the child prop type:\n{}",
        output.code
    );

    // The synthetic check identifier maps back to the authored attribute value.
    let value_start = template.find("You did it!").expect("static value present");
    let value_range = value_start..value_start + "You did it!".len();
    let sub_span = output
        .mappings
        .iter()
        .flat_map(|mapping| &mapping.sub_spans)
        .find(|span| span.src_range == value_range)
        .expect("static prop check should map to the authored attribute value");
    assert!(
        output.code[sub_span.gen_range.clone()].starts_with("__vize_prop_check_"),
        "synthetic check identifier should map to the static value: {sub_span:?}"
    );
}

#[test]
fn static_attribute_values_escape_into_exact_string_literals() {
    let script = r#"import Child from "./Child.vue"
"#;
    let template = "<Child label='say \"hi\" \\ done' />";

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
            .contains(": __Child_0_prop_label = \"say \\\"hi\\\" \\\\ done\";"),
        "static value must escape quotes and backslashes:\n{}",
        output.code
    );
}

#[test]
fn valueless_static_attributes_stay_out_of_per_prop_checks() {
    let script = r#"import Child from "./Child.vue"
"#;
    let template = r#"<Child disabled />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    assert!(
        !output.code.contains("__Child_0_prop_disabled ="),
        "valueless attributes keep their boolean-shorthand semantics:\n{}",
        output.code
    );
}

#[test]
fn repeated_prop_names_keep_unique_checks_and_one_type_alias() {
    let script = r#"import Child from "./Child.vue"
const isLoading = false
"#;
    let template = r#"<Child class="static-card" :class="{ loading: isLoading }" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    assert_eq!(
        output
            .code
            .matches("type __Child_0_prop_class = __VizePropValue<__Child_Props_0, 'class'>;")
            .count(),
        1,
        "the child prop type alias must be declared exactly once:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("const __vize_prop_check_0_class: __Child_0_prop_class = \"static-card\";"),
        "the static class value keeps the base check name:\n{}",
        output.code
    );
    assert!(
        output.code.contains(
            "const __vize_prop_check_0_class_2: __Child_0_prop_class = { loading: isLoading };"
        ),
        "the bound class value gets a unique check name:\n{}",
        output.code
    );
    assert_eq!(
        output
            .code
            .matches("const __vize_prop_check_0_class")
            .count(),
        2,
        "both authored values stay checked:\n{}",
        output.code
    );
}
