#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use super::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn generate(script: &str, template: &str) -> String {
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "template errors: {errors:?}");

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    generate_virtual_ts_with_offsets(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    )
    .code
    .to_string()
}

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
    let usages = vize_croquis::facts::component_usage_list(&summary);
    let usage = &usages[0];
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
        .mapping
        .semantic_links()
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
            .contains("void ({ [propName]: undefined }); // DynamicDirectiveArgument"),
        "dynamic prop names must be checked as expressions:\n{}",
        output.code
    );
    assert!(
        output
            .code
            .contains("void (eventName); // DynamicEventArgument"),
        "dynamic event names must be checked as expressions:\n{}",
        output.code
    );
}

#[test]
fn computed_component_event_names_never_become_static_listener_types() {
    let script = concat!(
        "import Child from './Child.vue'\n",
        "const Events = { UP: 'up' } as const\n",
        "const keys = Events\n",
        "const getKey = (name: string) => name\n",
        "const vnodeMounted = 'save'\n",
        "const enter = 'enter'\n",
        "let pressed = false\n",
    );
    for (target, argument) in [
        ("Child", "'ping'"),
        ("Child", r#"`${Events.UP}`"#),
        ("Child", "getKey(Events.UP)"),
        ("Child", "keys.UP"),
        ("Child", "(getKey(Events.UP))"),
        ("Child", "vnodeMounted"),
        ("button", "vnodeMounted"),
        ("Transition", "enter"),
    ] {
        let template = if target == "button" {
            format!(r#"<button @[{argument}]="pressed = true"></button>"#)
        } else {
            format!(r#"<{target} @[{argument}]="pressed = true" />"#)
        };
        let code = generate(script, &template);
        let allocator = oxc_allocator::Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &code, oxc_span::SourceType::ts()).parse();

        assert!(
            !parsed.panicked && parsed.diagnostics.is_empty(),
            "computed listener {argument} generated invalid TypeScript:\n{:?}\n{code}",
            parsed.diagnostics,
        );
        assert!(
            code.contains("// DynamicEventArgument"),
            "computed listener name must still be checked as an authored expression:\n{code}",
        );
        assert!(
            code.contains("type __vize_dynamic_component_"),
            "computed listener must use the open runtime listener contract:\n{code}",
        );
        assert!(
            !code.contains("type __vize_vnode_hook_"),
            "computed listener names must not be mistaken for static vnode hooks:\n{code}",
        );
    }
}
