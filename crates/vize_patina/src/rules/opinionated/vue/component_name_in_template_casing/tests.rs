use super::{ComponentCasing, ComponentNameInTemplateCasing, ComponentNameInTemplateCasingNuxt};
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(
        ComponentNameInTemplateCasing::default().with_registered_components_only(false),
    ));
    Linter::with_registry(registry)
}

#[test]
fn test_valid_pascal_case() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<MyComponent />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_kebab_case() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<my-component />"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_configured_kebab_case_allows_kebab_tags() {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(
        ComponentNameInTemplateCasing::new(ComponentCasing::KebabCase)
            .with_registered_components_only(false),
    ));
    let linter = Linter::with_registry(registry);
    let result = linter.lint_template(r#"<my-component />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_configured_kebab_case_reports_pascal_tags() {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(
        ComponentNameInTemplateCasing::new(ComponentCasing::KebabCase)
            .with_registered_components_only(false),
    ));
    let linter = Linter::with_registry(registry);
    let result = linter.lint_template(r#"<MyComponent />"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_valid_html_element() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div></div>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_vue_built_in() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<slot />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_nuxt_child_builtin() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<nuxt-child id="index" />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

fn create_nuxt_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ComponentNameInTemplateCasingNuxt::with_policy(
        ComponentNameInTemplateCasing::default().with_registered_components_only(false),
    )));
    Linter::with_registry(registry)
}

#[test]
fn test_nuxt_preset_allows_vuetify_tags() {
    let linter = create_nuxt_linter();
    let result = linter.lint_template(
        r#"<v-dialog><v-btn /><v-icon /><v-spacer /></v-dialog>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_all_tags_policy_still_flags_vuetify_tags() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<v-btn />"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_nuxt_preset_still_flags_other_kebab() {
    let linter = create_nuxt_linter();
    let result = linter.lint_template(r#"<my-component />"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}
