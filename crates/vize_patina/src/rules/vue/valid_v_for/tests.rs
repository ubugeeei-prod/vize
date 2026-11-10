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
fn rejects_empty_and_destructured_key_or_index_aliases() {
    let linter = create_linter();
    for expression in [
        "(, index) in items",
        "(value, , index) in record",
        "(item, { length }) in items",
        "(item, key, [index]) in items",
        "(item, index = 1) in items",
        "(value, key, index = 0) in record",
    ] {
        let source = format!(r#"<div v-for="{expression}" />"#);
        let result = linter.lint_template(&source, "test.vue");
        assert_eq!(
            result.error_count, 1,
            "{expression}: {:?}",
            result.diagnostics
        );
    }

    let valid = linter.lint_template(
        r#"<div v-for="({ id }, key, index) in items" :key="id" />"#,
        "test.vue",
    );
    assert_eq!(valid.error_count, 0, "{:?}", valid.diagnostics);
}

#[test]
fn accepts_line_breaks_around_in_and_of() {
    let linter = create_linter();
    for expression in [
        "item in\n  items",
        "item\n  in\n  items",
        "item\tof\titems",
        "(item,\n  index)\n  of items",
    ] {
        let source = format!(r#"<div v-for="{expression}" :key="item"></div>"#);
        let result = linter.lint_template(&source, "test.vue");
        assert_eq!(
            result.error_count, 0,
            "{expression}: {:?}",
            result.diagnostics
        );
    }
    // Vue requires whitespace before the keyword.
    let result = linter.lint_template(r#"<div v-for="(item)in items"></div>"#, "test.vue");
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
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
