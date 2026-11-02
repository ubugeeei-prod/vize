//! The actual drawer must preserve component ownership without capturing local slot/loop values.
use crate::drawer::{Drawer, DrawerOptions};
use crate::scope::ScopeData;
use vize_armature::parse;
use vize_carton::Allocator;

fn captured_target(script: &str, template: &str) -> bool {
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup(script);
    drawer.draw_template(&root);
    let summary = drawer.finish();
    let target = summary.scopes.iter().find_map(|scope| match scope.data() {
        ScopeData::EventHandler(data) if data.event_name == "select" => {
            data.target_component.as_deref()
        }
        _ => None,
    });
    let usage = summary
        .component_usages
        .iter()
        .find(|usage| usage.events.iter().any(|event| event.name == "select"));
    assert_eq!(target, usage.map(|usage| usage.name.as_str()));
    target.is_some_and(super::is_dynamic_component_alias)
}

#[test]
fn nested_script_owned_computed_target_keeps_its_component_event_owner() {
    let script = "import Child from './Child.vue'; const map = { foo: Child }; const key = 'foo';";
    assert!(captured_target(
        script,
        r#"<Panel><template #foo><component :is="map[key]" @select="value => value.toFixed()"></component></template></Panel>"#
    ));
    assert!(captured_target(
        script,
        r#"<div v-for="item in [1]"><component :is="map[key]" @select="value => value.toFixed()"></component></div>"#
    ));
}

#[test]
fn captured_local_unknown_and_type_only_roots_cannot_be_hoisted() {
    let script = "import Child from './Child.vue'; const map = { foo: Child }; const key = 'foo';";
    for template in [
        r#"<Panel><template #foo><component v-if="map" :is="map[key]" @select="value => value.toFixed()"></component></template></Panel>"#,
        r#"<Panel><template #foo><div v-if="map"><component :is="map[key]" @select="value => value.toFixed()"></component></div></template></Panel>"#,
        r#"<Panel><template #foo="{ key }"><component :is="map[key]" @select="value => value.toFixed()"></component></template></Panel>"#,
        r#"<Panel><template #foo="{ map }"><component :is="map[key]" @select="value => value.toFixed()"></component></template></Panel>"#,
        r#"<Panel><template #foo><component v-for="key in ['foo']" :is="map[key]" @select="value => value.toFixed()"></component></template></Panel>"#,
        r#"<Panel><template #foo><component :is="map[unknownKey]" @select="value => value.toFixed()"></component></template></Panel>"#,
        r#"<Panel><template #foo><component :is="globalThis[key]" @select="value => value.toFixed()"></component></template></Panel>"#,
    ] {
        assert!(!captured_target(script, template), "{template}");
    }
    assert!(!captured_target(
        "import type { map } from './map'; const key = 'foo';",
        r#"<Panel><template #foo><component :is="map[key]" @select="value => value.toFixed()"></component></template></Panel>"#
    ));
}

#[test]
fn script_member_uses_one_owned_capture_and_preserves_its_guard() {
    let allocator = Allocator::new();
    for (template, guard) in [
        (
            r#"<component :is="foo.bar" v-if="foo"></component>"#,
            Some("(foo)"),
        ),
        (
            r#"<template v-if="foo"><component :is="foo.bar"></component></template>"#,
            Some("(foo)"),
        ),
        (r#"<component :is="foo.bar"></component>"#, None),
    ] {
        let (root, errors) = parse(&allocator, template);
        assert!(errors.is_empty(), "{errors:?}");
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_script_setup("const foo = undefined as { bar: string } | undefined;");
        drawer.draw_template(&root);
        let summary = drawer.finish();
        assert_eq!(summary.component_usages.len(), 1);
        let usage = &summary.component_usages[0];
        assert!(super::is_dynamic_component_alias(usage.name.as_str()));
        assert_eq!(usage.vif_guard.as_deref(), guard);
        assert!(
            summary
                .template_expressions
                .iter()
                .any(|expression| expression.content == "foo.bar")
        );
    }
}

#[test]
fn new_computed_admission_does_not_shadow_an_existing_script_value() {
    let template = r#"<Panel><template #foo><component :is="map[key]" @select="value => value.toFixed()"></component></template></Panel>"#;
    let start = template.find("<component").expect("authored component") as u32;
    let alias = super::dynamic_component_alias(start);
    let script = vize_carton::cstr!(
        "import Child from './Child.vue'; const map = {{ foo: Child }}; const key = 'foo'; const {alias} = 1;"
    );
    assert!(!captured_target(script.as_str(), template));
}
