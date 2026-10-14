//! Cases mirror eslint-plugin-vue's `valid-v-is` suite. Templates are written
//! at the `lint_template` call so the markup differential battery picks them
//! up as fixtures.

use super::ValidVIs;
use crate::linter::{LintResult, Linter};
use crate::rule::RuleRegistry;

fn linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVIs));
    Linter::with_registry(registry)
}

fn messages(result: &LintResult) -> Vec<&str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.rule_name, "vue/valid-v-is");
            diagnostic.message.as_str()
        })
        .collect()
}

fn reported<'a>(source: &'a str, result: &LintResult) -> Vec<&'a str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            source
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default()
        })
        .collect()
}

const EXPECTED_VALUE: &str = "'v-is' directives require that attribute value.";

#[test]
fn accepts_native_element_with_value() {
    let result = linter().lint_template(r#"<div v-is="foo" />"#, "test.vue");
    assert!(messages(&result).is_empty());
}

#[test]
fn accepts_table_row_with_value() {
    let result = linter().lint_template(r#"<table><tr v-is="'MyRow'"></tr></table>"#, "test.vue");
    assert!(messages(&result).is_empty());
}

#[test]
fn accepts_svg_owner() {
    let result = linter().lint_template(r#"<svg><g v-is="foo" /></svg>"#, "test.vue");
    assert!(messages(&result).is_empty());
}

#[test]
fn ignores_the_is_attribute() {
    let result = linter().lint_template(r#"<MyComponent is="vue:Other" />"#, "test.vue");
    assert!(messages(&result).is_empty());
}

#[test]
fn reports_argument_at_the_argument() {
    let source = r#"<div v-is:a="foo" />"#;
    let result = linter().lint_template(r#"<div v-is:a="foo" />"#, "test.vue");
    assert_eq!(
        messages(&result),
        ["'v-is' directives require no argument."]
    );
    assert_eq!(reported(source, &result), ["a"]);
}

#[test]
fn reports_all_modifiers_once() {
    let source = r#"<div v-is.a.b="foo" />"#;
    let result = linter().lint_template(r#"<div v-is.a.b="foo" />"#, "test.vue");
    assert_eq!(
        messages(&result),
        ["'v-is' directives require no modifier."]
    );
    assert_eq!(reported(source, &result), ["a.b"]);
}

#[test]
fn reports_missing_value() {
    let result = linter().lint_template(r#"<div v-is />"#, "test.vue");
    assert_eq!(messages(&result), [EXPECTED_VALUE]);
}

#[test]
fn reports_empty_value() {
    let result = linter().lint_template(r#"<div v-is="" />"#, "test.vue");
    assert_eq!(messages(&result), [EXPECTED_VALUE]);
}

#[test]
fn reports_component_owner() {
    let result = linter().lint_template(r#"<MyComponent v-is="foo" />"#, "test.vue");
    assert_eq!(
        messages(&result),
        ["'v-is' directive must be owned by a native HTML element, but 'MyComponent' is not."]
    );
}

#[test]
fn reports_custom_element_owner() {
    let result = linter().lint_template(r#"<my-element v-is="foo" />"#, "test.vue");
    assert_eq!(
        messages(&result),
        ["'v-is' directive must be owned by a native HTML element, but 'my-element' is not."]
    );
}
