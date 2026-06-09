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
fn test_vize_expected_suppresses_every_diagnostic_on_the_line() {
    // Regression for #968: `@vize:expected` was implemented with `remove`
    // and so consumed itself after the first matching diagnostic — any
    // additional diagnostic on the same line then leaked through. The
    // directive must scope to the whole line.
    let linter = Linter::new();
    let result = linter.lint_template(
        r#"<!-- @vize:expected -->
<li v-for="item in items" v-if="item.active">{{ item }}</li>"#,
        "test.vue",
    );
    assert_eq!(
        result.error_count, 0,
        "v-for missing-key error should be suppressed"
    );
    assert_eq!(
        result.warning_count, 0,
        "v-if-with-v-for warning on the same line must also be suppressed"
    );
}

#[test]
fn test_vize_expected_suppresses_run_on_template_diagnostic() {
    // Regression for #968: `run_on_template` rules emit during a phase
    // that runs *before* per-element traversal would register directives,
    // so suppression directives must be pre-scanned. `vue/no-dupe-v-else-if`
    // reports in that phase.
    let linter = Linter::new();
    let result = linter.lint_template(
        r#"<!-- @vize:expected -->
<div v-if="a"></div><div v-else-if="a"></div>"#,
        "test.vue",
    );
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|d| d.rule_name == "vue/no-dupe-v-else-if")
            .count(),
        0,
        "run_on_template-phase rule must be suppressed by @vize:expected"
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
fn test_vize_ignore_start_end_suppresses_run_on_template_diagnostic() {
    // Regression for #1196: `@vize:ignore-start` / `@vize:ignore-end`
    // regions must already be known before `run_on_template` rules report.
    let linter = Linter::new();
    let result = linter.lint_template(
        r#"<!-- @vize:ignore-start -->
<div v-if="a"></div>
<div v-else-if="a"></div>
<!-- @vize:ignore-end -->"#,
        "test.vue",
    );
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|d| d.rule_name == "vue/no-dupe-v-else-if")
            .count(),
        0,
        "run_on_template-phase rule must be suppressed by @vize:ignore-start/end"
    );
}

#[test]
fn test_vize_forget_suppresses_run_on_template_diagnostic() {
    // Regression for #1196: `@vize:forget` suppresses the next template
    // child, including the branches in its v-if / v-else-if chain.
    let linter = Linter::new();
    let result = linter.lint_template(
        r#"<!-- @vize:forget duplicate condition is intentional -->
<div v-if="a"></div>
<div v-else-if="a"></div>"#,
        "test.vue",
    );
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|d| d.rule_name == "vue/no-dupe-v-else-if")
            .count(),
        0,
        "run_on_template-phase rule must be suppressed by @vize:forget"
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
