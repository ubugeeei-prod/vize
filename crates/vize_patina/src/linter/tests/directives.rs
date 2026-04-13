use super::Linter;

#[test]
fn test_vize_todo_emits_warning() {
    let linter = Linter::new();
    let result = linter.lint_template(
        r#"<div><!-- @vize:todo fix this --><span>hello</span></div>"#,
        "test.vue",
    );
    assert_eq!(
        result.warning_count, 1,
        "Should emit 1 warning for @vize:todo"
    );
    assert_eq!(result.diagnostics[0].rule_name, "vize/todo");
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_vize_fixme_emits_error() {
    let linter = Linter::new();
    let result = linter.lint_template(
        r#"<div><!-- @vize:fixme broken --><span>hello</span></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1, "Should emit 1 error for @vize:fixme");
    assert_eq!(result.diagnostics[0].rule_name, "vize/fixme");
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_vize_deprecated_emits_warning() {
    let linter = Linter::new();
    let result = linter.lint_template(
        r#"<div><!-- @vize:deprecated use NewComp --><span>hello</span></div>"#,
        "test.vue",
    );
    assert_eq!(
        result.warning_count, 1,
        "Should emit 1 warning for @vize:deprecated"
    );
    assert_eq!(result.diagnostics[0].rule_name, "vize/deprecated");
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_vize_expected_suppresses_error() {
    let linter = Linter::new();
    let result = linter.lint_template(
        r#"<ul><li v-for="item in items">{{ item }}</li></ul>"#,
        "test.vue",
    );
    assert!(result.error_count > 0, "Should have error without key");

    let result = linter.lint_template(
        r#"<ul><!-- @vize:expected -->
<li v-for="item in items">{{ item }}</li></ul>"#,
        "test.vue",
    );
    assert_eq!(
        result.error_count, 0,
        "Error should be suppressed by @vize:expected"
    );
}

#[test]
fn test_vize_ignore_start_end_region() {
    let linter = Linter::new();
    let result = linter.lint_template(
        r#"<!-- @vize:ignore-start -->
<ul><li v-for="item in items">{{ item }}</li></ul>
<!-- @vize:ignore-end -->"#,
        "test.vue",
    );
    assert_eq!(
        result.error_count, 0,
        "Errors in ignore region should be suppressed"
    );
}

#[test]
fn test_vize_docs_no_lint_effect() {
    let linter = Linter::new();
    let result = linter.lint_template(
        r#"<div><!-- @vize:docs Component documentation --><span>hello</span></div>"#,
        "test.vue",
    );
    assert_eq!(
        result.error_count, 0,
        "Docs directive should not produce errors"
    );
    assert_eq!(
        result.warning_count, 0,
        "Docs directive should not produce warnings"
    );
}
