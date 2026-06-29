use super::ValidVModel;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVModel));
    Linter::with_registry(registry)
}

#[test]
fn invalid_v_model_call_expression() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<input v-model="foo()">"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn invalid_v_model_binary_expression() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<input v-model="foo + bar">"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn invalid_v_model_optional_chain() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<input v-model="foo?.bar">"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn invalid_v_model_this_expression() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<input v-model="this">"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn invalid_v_model_with_trailing_tokens() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<input v-model="foo;bar">"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn valid_v_model_member_targets() {
    let linter = create_linter();
    for source in [
        r#"<input v-model="foo.bar">"#,
        r#"<input v-model="foo[bar]">"#,
        r#"<input v-model="foo().bar">"#,
        r#"<input v-model="(foo)">"#,
    ] {
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.error_count, 0, "{source}");
    }
}
