//! Cases mirror eslint-plugin-vue's `no-deprecated-v-is` suite. Templates are
//! written at the `lint_template` call so the markup differential battery
//! picks them up as fixtures.

use super::NoDeprecatedVIs;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoDeprecatedVIs));
    Linter::with_registry(registry)
}

#[test]
fn accepts_is_attribute_with_vue_prefix() {
    let result = linter().lint_template(r#"<div is="vue:MyComponent" />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn accepts_dynamic_component_shorthand() {
    let result = linter().lint_template(r#"<component :is="MyComponent" />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn accepts_dynamic_component_longhand() {
    let result = linter().lint_template(r#"<component v-bind:is="MyComponent" />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn reports_v_is_at_the_directive_key() {
    let source = r#"<div v-is="'MyComponent'" />"#;
    let result = linter().lint_template(r#"<div v-is="'MyComponent'" />"#, "test.vue");
    assert_eq!(result.warning_count, 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, "vue/no-deprecated-v-is");
    assert_eq!(diagnostic.message, "`v-is` directive is deprecated.");
    assert_eq!(
        source.get(diagnostic.start as usize..diagnostic.end as usize),
        Some("v-is")
    );
}

#[test]
fn reports_v_is_without_value() {
    let result = linter().lint_template(r#"<div v-is />"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn reports_each_v_is() {
    let result = linter().lint_template(
        r#"<table><tr v-is="'A'"></tr><tr v-is="'B'"></tr></table>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 2);
}
