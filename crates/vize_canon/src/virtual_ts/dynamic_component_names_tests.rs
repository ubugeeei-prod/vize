use super::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn dynamic_prop_names_do_not_become_static_component_contract_keys() {
    let script = concat!(
        "import Child from './Child.vue'\n",
        "const staticValue = 'fixed'\n",
        "const dynamicValue = 'runtime'\n",
        "const propName = 'staticProp'\n",
        "const eventName = 'save'\n",
        "const handler = () => {}\n",
    );
    let template = concat!(
        "<Child :static-prop=\"staticValue\" ",
        ":[propName]=\"dynamicValue\" ",
        "@save=\"handler\" @[eventName]=\"handler\" />",
    );
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "template errors: {errors:?}");

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let usage = &summary.component_usages[0];
    assert!(!usage.props[0].name_is_dynamic);
    assert!(usage.props[1].name_is_dynamic);
    assert!(!usage.events[0].name_is_dynamic);
    assert!(usage.events[1].name_is_dynamic);

    let output = generate_virtual_ts_with_offsets(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    );
    let prop_link = output
        .semantic_links
        .iter()
        .find(|link| link.kind == super::VizeSemanticLinkKind::VueComponentPropNavigation)
        .expect("component prop navigation link");
    assert_eq!(&output.code[prop_link.source_range.clone()], "Child");
    assert_eq!(&output.code[prop_link.target_range.clone()], "staticProp");

    assert!(output.code.contains("__Child_0_prop_static_prop"));
    assert!(output.code.contains("\"staticProp\": staticValue"));
    assert!(!output.code.contains("__Child_0_prop_propName"));
    assert!(!output.code.contains("\"propName\": dynamicValue"));
    assert!(!output.code.contains("__vize_props_nav_0.propName"));
    assert!(
        output.code.contains("void (dynamicValue); // VBind"),
        "dynamic value expressions must still be checked:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("void (propName); // DynamicDirectiveArgument"),
        "dynamic prop names must be checked as expressions:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("void (eventName); // DynamicDirectiveArgument"),
        "dynamic event names must be checked as expressions:\n{}",
        output.code
    );
}
