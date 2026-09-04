use super::NoArrayIndexKey;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoArrayIndexKey));
    Linter::with_registry(registry)
}

#[test]
fn reports_index_used_as_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="(item, index) in list" :key="index">{{ item }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn allows_dynamic_key_argument() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="(item, index) in list" :[key]="index">{{ item }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn reports_object_iteration_index_used_as_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="(value, key, index) in obj" :key="index">{{ value }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn reports_index_with_spaces_in_key() {
    let linter = create_linter();
    // `:key=" index "` is still just the index identifier.
    let result = linter.lint_template(
        r#"<li v-for="(item, index) in list" :key=" index ">{{ item }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn reports_v_bind_key_long_form() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="(item, index) in list" v-bind:key="index">{{ item }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn reports_template_v_for_child_index_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<template v-for="(item, index) in list"><li :key="index">{{ item }}</li></template>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn allows_template_v_for_child_stable_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<template v-for="(item, index) in list"><li :key="item.id">{{ item }}</li></template>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn allows_stable_id_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="(item, index) in list" :key="item.id">{{ item }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn allows_index_composed_into_key() {
    let linter = create_linter();
    // Using the index as part of a larger key string is not a bare index.
    let result = linter.lint_template(
        r#"<li v-for="(item, index) in list" :key="`row-${index}`">{{ item }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn allows_object_3tuple_key_used_as_key() {
    // For object iteration `(value, key, index)`, the second binding is the
    // stable object key, so `:key="key"` is fine. Only the third binding is the
    // positional counter.
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="(value, key, index) in obj" :key="key">{{ value }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn allows_no_index_alias() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="item in list" :key="item.id">{{ item }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn ignores_index_like_identifier_that_is_not_the_alias() {
    // `idx` is the v-for index; `:key="index"` references some unrelated outer
    // `index`, not the loop index, so it must not be flagged.
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="(item, idx) in list" :key="index">{{ item }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn allows_of_delimiter_with_stable_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="(item, index) of list" :key="item.id">{{ item }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn reports_of_delimiter_index_as_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="(item, index) of list" :key="index">{{ item }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn ignores_object_destructuring_value_used_as_key() {
    // `{ id }` destructures the value; `id` is not a positional index.
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<li v-for="{ id } in list" :key="id">{{ id }}</li>"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 0);
}
