use super::generate_virtual_ts;

#[test]
fn component_template_navigation_mappings() {
    use vize_croquis::{Analyzer, AnalyzerOptions};

    let script = r#"import Child from './Child.vue'
const count = 1
"#;
    let template = r#"<Child label="ready" :count="count" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    let tag_start = template.find("Child").unwrap();
    let tag_mapping = output
        .mappings
        .iter()
        .find(|mapping| mapping.src_range == (tag_start..tag_start + "Child".len()))
        .expect("component tag should map to a generated component reference");
    assert_eq!(&output.code[tag_mapping.gen_range.clone()], "Child");

    let static_prop_start = template.find("label").unwrap();
    let static_prop_mapping = output
        .mappings
        .iter()
        .find(|mapping| mapping.src_range == (static_prop_start..static_prop_start + "label".len()))
        .expect("static prop name should map to a generated prop type reference");
    assert_eq!(&output.code[static_prop_mapping.gen_range.clone()], "label");

    let dynamic_prop_start = template.find(":count").unwrap() + 1;
    let dynamic_prop_mapping = output
        .mappings
        .iter()
        .find(|mapping| {
            mapping.src_range == (dynamic_prop_start..dynamic_prop_start + "count".len())
        })
        .expect("dynamic prop name should map to a generated prop type reference");
    assert_eq!(
        &output.code[dynamic_prop_mapping.gen_range.clone()],
        "count"
    );
}
