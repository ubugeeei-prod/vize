use crate::drawer::{Drawer, DrawerOptions};

#[test]
fn dynamic_component_props_use_is_binding_as_target_component() {
    use vize_armature::parse;
    use vize_carton::Bump;

    let template = r#"<component :is="Child" :count="count"></component>"#;

    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "Template should parse without errors");

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_template(&root);
    let summary = drawer.finish();
    let usage = summary
        .component_usages
        .iter()
        .find(|usage| usage.name == "Child")
        .expect("dynamic component should create a usage for the :is target");

    assert_eq!(
        usage
            .props
            .iter()
            .map(|prop| prop.name.as_str())
            .collect::<Vec<_>>(),
        ["count"],
        "the runtime-only is binding must not be checked as a child prop"
    );
}

#[test]
fn dynamic_component_lowercase_is_value_does_not_create_component_usage() {
    use vize_armature::parse;
    use vize_carton::Bump;

    let template = r#"<component :is="as" :class="klass"></component>"#;

    let allocator = Bump::new();
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "Template should parse without errors");

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_template(&root);
    let summary = drawer.finish();

    assert!(
        summary.component_usages.is_empty(),
        "an unresolved lowercase :is value is not a concrete component usage"
    );
    assert!(
        !summary.used_components.contains("as"),
        "the unresolved lowercase :is value must not create an auto-import stub"
    );
}
