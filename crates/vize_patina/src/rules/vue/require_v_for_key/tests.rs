use super::RequireVForKey;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(RequireVForKey));
    Linter::with_registry(registry)
}

#[test]
fn test_valid_v_for_with_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<ul><li v-for="item in items" :key="item.id">{{ item.name }}</li></ul>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_v_for_without_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<ul><li v-for="item in items">{{ item.name }}</li></ul>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_valid_v_for_with_static_key() {
    let linter = create_linter();
    // Static key is unusual but technically valid.
    let result = linter.lint_template(
        r#"<div v-for="item in items" key="static"></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_slot_outlet_v_for_without_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div><slot v-for="(item, index) in items" name="item" :item="item" :index="index" /></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_petite_vue_keyless_v_for_allowed() {
    let linter = create_linter();
    // petite-vue allows keyless v-for.
    let result = linter.lint_standalone_html(
        r#"<!DOCTYPE html>
<html>
  <body>
    <ul v-scope="{ items: [1, 2, 3] }">
      <li v-for="item in items">{{ item }}</li>
    </ul>
    <script src="https://unpkg.com/petite-vue" init></script>
  </body>
</html>"#,
        "index.html",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_non_petite_html_keyless_v_for_still_reports() {
    let linter = create_linter();
    // A plain HTML document keeps the Vue 3 requirement.
    let result = linter.lint_standalone_html(
        r#"<!DOCTYPE html>
<html>
  <body>
    <ul>
      <li v-for="item in items">{{ item }}</li>
    </ul>
    <script src="https://unpkg.com/vue"></script>
  </body>
</html>"#,
        "index.html",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_template_v_for_child_key_is_not_missing_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<template v-for="item in items"><div :key="item.id">{{ item }}</div></template>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_template_v_for_dynamic_slot_forwarder_is_not_missing_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<Component>
  <template v-for="header in headers" #[`header.${header.value}`]>
    <slot :name="`header.${header.value}`" :header="header" />
  </template>
</Component>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}
