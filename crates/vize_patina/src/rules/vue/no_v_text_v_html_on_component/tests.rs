use super::NoVTextVHtmlOnComponent;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoVTextVHtmlOnComponent));
    Linter::with_registry(registry)
}

#[test]
fn test_valid_v_html_on_div() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div v-html="content"></div>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_text_on_span() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<span v-text="content"></span>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_v_html_on_component() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<MyComponent v-html="content" />"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_v_html_on_kebab_case_component() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<my-component v-html="content" />"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_v_html_on_dynamic_component_bound_to_tag_name_prop() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script setup lang="ts">
const { tagName = "div", html = "" } = defineProps<{
  tagName?: string
  html?: string
}>()
</script>

<template>
  <component :is="tagName" v-html="html" />
</template>"#,
        "PreviewCard.vue",
    );
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_valid_v_html_on_dynamic_component_bound_to_tag_name_identifier() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<component :is="tagName" v-html="content" />"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_valid_v_html_on_dynamic_component_with_native_string_literal_binding() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<component :is="'article'" v-html="content" />"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn test_invalid_v_html_on_dynamic_component_bound_to_component_reference() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<component :is="SomeComponent" v-html="content" />"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_v_html_on_dynamic_component_bound_to_component_carrier() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<component :is="currentComponent" v-html="content" />"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_v_html_on_dynamic_component_with_component_string_literal_binding() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<component :is="'MyComponent'" v-html="content" />"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_v_text_on_component() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<MyComponent v-text="content" />"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_component_with_slot_content() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<MyComponent>{{ content }}</MyComponent>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

/// Exact reproduction of #3211: a static `is="div"` resolves to a native
/// `<div>`, so `v-html` must be allowed exactly as on a literal `<div>`.
#[test]
fn test_valid_v_html_on_component_with_static_native_is_sfc() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script setup lang="ts">
defineProps<{ html: string }>();
</script>

<template>
  <component is="div" v-html="html" />
</template>"#,
        "App.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_text_on_component_with_static_is_span() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<component is="span" v-text="content" />"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_html_on_component_with_static_is_svg_tag() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<component is="svg" v-html="content" />"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_html_on_uppercase_component_tag_with_static_native_is() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<Component is="div" v-html="content" />"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_v_html_on_component_with_static_component_is() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<component is="MyComponent" v-html="content" />"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_v_html_on_component_with_unknown_is() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<component is="unknown-thing" v-html="content" />"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_v_html_on_component_without_is() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<component v-html="content" />"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

/// Vue strips the `vue:` prefix only on literal native tags; on `<component>`
/// the value reaches `resolveDynamicComponent` verbatim and does not render a
/// native `<div>`, so the directive stays flagged.
#[test]
fn test_invalid_v_html_on_component_with_vue_prefixed_is() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<component is="vue:div" v-html="content" />"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

/// A `:is` binding wins over a static `is` attribute in codegen, so a
/// component-shaped binding stays flagged even when the static value is native.
#[test]
fn test_invalid_v_html_when_component_bind_is_shadows_static_native_is() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<component is="div" :is="SomeComponent" v-html="content" />"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_v_html_on_uppercase_component_tag_without_is() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<Component v-html="content" />"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

/// On a literal native element `is` means customized built-in elements, not
/// component substitution -- behavior there must not change.
#[test]
fn test_valid_v_html_on_native_element_with_is_attribute() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div is="my-thing" v-html="content" />"#, "test.vue");
    assert_eq!(result.error_count, 0);
}
