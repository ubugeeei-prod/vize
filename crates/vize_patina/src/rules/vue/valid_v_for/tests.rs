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
