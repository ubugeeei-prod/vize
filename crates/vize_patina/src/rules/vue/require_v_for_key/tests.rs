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
fn test_static_key_does_not_satisfy_v_for() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div v-for="item in items" key="static"></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);

    let transparent = linter.lint_template(
        r#"<template v-for="item in items" key="item.id"><div /></template>"#,
        "test.vue",
    );
    assert_eq!(transparent.error_count, 1);
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
fn test_template_v_for_reports_each_unkeyed_native_child() {
    let result = create_linter().lint_template(
        r#"<svg>
  <template v-for="point in points">
    <circle :cx="point.x" />
    <text>{{ point.label }}</text>
  </template>
</svg>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 2, "{:?}", result.diagnostics);
}

#[test]
fn test_multiline_native_v_for_without_key_reports() {
    let result = create_linter().lint_template(
        r#"<svg>
  <circle
    class="point"
    v-for="point in points"
    :cx="point.x"
  />
</svg>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
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

#[test]
fn dynamic_slot_v_for_requires_a_key_for_native_slot_content() {
    let result = create_linter().lint_template(
        r#"<Component>
  <template v-for="name in names" #[name]="slotProps">
    <span v-if="slotProps.visible" />
  </template>
</Component>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn nested_template_scopes_keep_native_v_for_checks() {
    let result = create_linter().lint_template(
        r#"<svg>
  <template v-for="series in allSeries">
    <template v-if="show">
      <circle v-for="point in series.points" :cx="point.x" />
    </template>
  </template>
</svg>"#,
        "test.vue",
    );
    assert!(result.error_count >= 1, "{:?}", result.diagnostics);
}
