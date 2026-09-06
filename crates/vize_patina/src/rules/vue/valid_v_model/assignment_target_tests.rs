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
fn invalid_v_model_conditional_expression_reports_expression_range() {
    let linter = create_linter();
    let source = r#"<el-input v-model="multiple ? presentText : inputValue" />"#;
    let result = linter.lint_template(source, "test.vue");
    assert_eq!(result.error_count, 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "multiple ? presentText : inputValue"
    );
}

#[test]
fn invalid_component_v_model_argument_reports_expression_range() {
    let linter = create_linter();
    let source = r#"<Dialog v-model:open="confirmDeleteKey !== null" />"#;
    let result = linter.lint_template(source, "test.vue");
    assert_eq!(result.error_count, 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "confirmDeleteKey !== null"
    );
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
fn invalid_v_model_malformed_expression() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<input v-model="foo(">"#, "test.vue");
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
        r#"<AppSelect v-model="(cell?.data as Permission).permission" />"#,
    ] {
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.error_count, 0, "{source}");
    }
}
