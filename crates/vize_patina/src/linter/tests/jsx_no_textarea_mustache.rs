use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::vue::NoTextareaMustache;
use vize_atelier_jsx::JsxLang;

fn linter_with(rule: Box<dyn Rule>) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry)
}

fn diagnostic_rules(result: &LintResult) -> Vec<&str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name.as_ref())
        .collect()
}

fn diagnostic_slices<'a>(source: &'a str, result: &LintResult) -> Vec<&'a str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| &source[diagnostic.start as usize..diagnostic.end as usize])
        .collect()
}

#[test]
fn no_textarea_mustache_fires_on_jsx_and_tsx_markup() {
    let linter = linter_with(Box::new(NoTextareaMustache));
    let source = r#"const A = () => <textarea>{message}</textarea>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(diagnostic_rules(&result), vec!["vue/no-textarea-mustache"]);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);
    assert_eq!(diagnostic_slices(source, &result), vec![r#"{message}"#]);

    let diag = &result.diagnostics[0];
    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(
        diag.message,
        "Mustache interpolation inside textarea is not allowed"
    );
    assert_eq!(
        diag.help.as_deref(),
        Some("Use v-model instead of mustache interpolation in textarea")
    );
    assert!(diag.fix.is_none());

    let tsx = r#"const A = (): JSX.Element => <textarea>{message}</textarea>;"#;
    let tsx_result = linter.lint_jsx(tsx, "test.tsx", JsxLang::Tsx);
    assert_eq!(
        diagnostic_slices(tsx, &tsx_result),
        vec![r#"{message}"#],
        "TSX should use the same authored source range"
    );
}

#[test]
fn no_textarea_mustache_preserves_jsx_clean_boundaries() {
    let linter = linter_with(Box::new(NoTextareaMustache));
    for source in [
        r#"const A = () => <textarea defaultValue={message} />;"#,
        r#"const A = () => <div>{message}</div>;"#,
        r#"const A = () => <Textarea>{message}</Textarea>;"#,
        r#"const A = () => <TEXTAREA>{message}</TEXTAREA>;"#,
        r#"const A = () => <Forms.textarea>{message}</Forms.textarea>;"#,
        r#"const A = () => <svg:textarea>{message}</svg:textarea>;"#,
        r#"const A = () => <textarea><span>{message}</span></textarea>;"#,
        r#"const A = () => <textarea>{/* comment */}</textarea>;"#,
        r#"const A = () => <textarea>{"message"}</textarea>;"#,
        r#"const A = () => <textarea>{cond && <span />}</textarea>;"#,
        r#"const A = () => <textarea>{condition ? <span /> : <em />}</textarea>;"#,
        r#"const A = () => <textarea>{items.map(() => <span />)}</textarea>;"#,
        r#"const A = () => <Comp fallback={<textarea>{message}</textarea>} />;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.error_count, 0,
            "must stay clean for {source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.warning_count, 0, "must not warn for {source}");
    }
}

#[test]
fn no_textarea_mustache_reports_multiple_jsx_expressions_in_source_order() {
    let linter = linter_with(Box::new(NoTextareaMustache));
    let source = r#"const A = () => <textarea>{first}{second}</textarea>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["vue/no-textarea-mustache", "vue/no-textarea-mustache"]
    );
    assert_eq!(result.error_count, 2);
    assert_eq!(result.warning_count, 0);
    assert_eq!(
        diagnostic_slices(source, &result),
        vec![r#"{first}"#, r#"{second}"#]
    );
}

#[test]
fn no_textarea_mustache_reports_direct_jsx_expression_shapes() {
    let linter = linter_with(Box::new(NoTextareaMustache));
    for (source, expected) in [
        (
            r#"const A = () => <textarea>{a + b}</textarea>;"#,
            r#"{a + b}"#,
        ),
        (
            r#"const A = () => <textarea>{null}</textarea>;"#,
            r#"{null}"#,
        ),
        (r#"const A = () => <textarea>{0}</textarea>;"#, r#"{0}"#),
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(diagnostic_rules(&result), vec!["vue/no-textarea-mustache"]);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.warning_count, 0);
        assert_eq!(diagnostic_slices(source, &result), vec![expected]);
    }
}

#[test]
fn no_textarea_mustache_keeps_jsx_template_literal_as_expression() {
    let linter = linter_with(Box::new(NoTextareaMustache));
    let source = r#"const A = () => <textarea>{`message`}</textarea>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(diagnostic_rules(&result), vec!["vue/no-textarea-mustache"]);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);
    assert_eq!(diagnostic_slices(source, &result), vec![r#"{`message`}"#]);
}

#[test]
fn no_textarea_mustache_reports_jsx_fragment_children_after_lowering() {
    let linter = linter_with(Box::new(NoTextareaMustache));
    let source = r#"const A = () => <textarea><>{message}</></textarea>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(diagnostic_rules(&result), vec!["vue/no-textarea-mustache"]);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);
    assert_eq!(diagnostic_slices(source, &result), vec![r#"{message}"#]);
}

#[test]
fn migrated_no_textarea_mustache_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(NoTextareaMustache));
    let result = linter.lint_jsx(
        r#"const A = () => <textarea>{message}</textarea>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated no-textarea-mustache rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn no_textarea_mustache_honors_jsx_lowered_markup_rule_config() {
    let source = r#"const A = () => <textarea>{message}</textarea>;"#;
    let warning = linter_with(Box::new(NoTextareaMustache))
        .with_rule_severity_overrides(vec![("vue/no-textarea-mustache".into(), Severity::Warning)])
        .lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(warning.error_count, 0);
    assert_eq!(warning.warning_count, 1);
    assert_eq!(warning.diagnostics[0].severity, Severity::Warning);

    let disabled = linter_with(Box::new(NoTextareaMustache))
        .with_disabled_rules(vec!["vue/no-textarea-mustache".into()])
        .lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert!(
        disabled.diagnostics.is_empty(),
        "disabled lowered-markup rules must not report: {:?}",
        disabled.diagnostics
    );
}

#[test]
fn no_textarea_mustache_sfc_offsets_template_diagnostics() {
    let source = r#"<script setup lang="ts">
const message = "hello";
</script>

<template>
  <textarea>{{ message }}</textarea>
</template>
"#;
    let linter = linter_with(Box::new(NoTextareaMustache));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(diagnostic_rules(&result), vec!["vue/no-textarea-mustache"]);
    assert_eq!(diagnostic_slices(source, &result), vec![r#"{{ message }}"#]);

    let expected = source.find(r#"{{ message }}"#).unwrap() as u32;
    let diag = &result.diagnostics[0];
    assert_eq!(
        diag.start, expected,
        "SFC diagnostic must be offset into file coordinates"
    );
    assert_eq!(diag.end, expected + r#"{{ message }}"#.len() as u32);
}

#[test]
fn no_textarea_mustache_sfc_does_not_lint_script_tsx() {
    let source = r#"<script setup lang="tsx">
const A = () => <textarea>{message}</textarea>;
</script>
"#;
    let linter = linter_with(Box::new(NoTextareaMustache));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        result.error_count, 0,
        "SFC no-textarea-mustache must not inspect JSX inside script blocks: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 0);
}
