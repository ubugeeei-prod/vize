use super::ValidVFor;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVFor));
    Linter::with_registry(registry)
}

#[test]
fn test_valid_v_for() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div v-for="item in items" :key="item.id"></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_for_with_index() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div v-for="(item, index) in items" :key="index"></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_for_of() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div v-for="item of items" :key="item.id"></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_v_for_no_expression() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div v-for></div>"#, "test.vue");
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_v_for_empty_expression() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div v-for=""></div>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_v_for_no_in_or_of() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div v-for="items"></div>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn dynamic_slot_templates_do_not_require_keys_on_component_bodies() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyList>
            <template v-for="name in names" #[name]="scope">
                <component :is="slots[name]" v-bind="scope" />
            </template>
            <template v-for="name in names" #[name]>
                <MyItem :label="name" />
            </template>
        </MyList>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn ordinary_template_iterations_still_require_keys_on_component_bodies() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<template v-for="item in items"><MyItem :label="item" /></template>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn template_iterations_require_keys_on_slot_bodies() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<template v-for="item in items"><slot :item="item"><span>{{ item }}</span></slot></template>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn template_iterations_accept_slot_fallbacks_owned_by_keyed_nested_loops() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<template v-for="i in count"><slot><MyItem v-for="item in rows" :key="`${i}-${item}`" /></slot></template>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn template_iterations_accept_keyed_slot_fallback_roots() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<template v-for="(option, index) in options"><slot name="tag"><span :key="index">{{ option }}</span></slot></template>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn directly_iterated_slots_remain_exempt_from_keys() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<slot v-for="item in items" :item="item"><span :key="item.id" /></slot>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}
