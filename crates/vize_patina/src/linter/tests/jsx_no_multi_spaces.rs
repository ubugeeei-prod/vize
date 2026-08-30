use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::vue::NoMultiSpaces;
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
fn no_multi_spaces_fires_on_jsx_and_tsx_markup() {
    let linter = linter_with(Box::new(NoMultiSpaces::default()));
    let source = r#"const A = () => <div className="foo"  id="bar" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(diagnostic_rules(&result), vec!["vue/no-multi-spaces"]);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
    assert_eq!(diagnostic_slices(source, &result), vec!["  "]);

    let diag = &result.diagnostics[0];
    assert_eq!(diag.severity, Severity::Warning);
    assert_eq!(diag.message, "Multiple consecutive spaces");
    assert_eq!(diag.help, None);
    let fix = diag
        .fix
        .as_ref()
        .expect("no-multi-spaces must stay fixable");
    assert_eq!(fix.message, "Replace multiple spaces with single space");
    assert_eq!(fix.edits.len(), 1);
    assert_eq!(fix.edits[0].start, diag.start);
    assert_eq!(fix.edits[0].end, diag.end);
    assert_eq!(fix.edits[0].new_text, " ");

    let tsx = r#"const A = (): JSX.Element => <div  data-id="bar" />;"#;
    let tsx_result = linter.lint_jsx(tsx, "test.tsx", JsxLang::Tsx);
    assert_eq!(
        diagnostic_slices(tsx, &tsx_result),
        vec!["  "],
        "TSX should use the same authored source gap"
    );
}

#[test]
fn no_multi_spaces_preserves_jsx_clean_boundaries() {
    let linter = linter_with(Box::new(NoMultiSpaces::default()));
    for source in [
        r#"const A = () => <div className="foo" id="bar" />;"#,
        r#"const A = () => <div className="foo" />;"#,
        r#"const A = () => <div />;"#,
        r#"const A = () => <div {...props} />;"#,
        r#"const A = () => <div prop={{ spaced: true }} id="bar" />;"#,
        r#"const A = () => <button
  className="btn"
  disabled={isDisabled}
/>;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 0,
            "must stay clean for {source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.error_count, 0, "must not error for {source}");
    }
}

#[test]
fn no_multi_spaces_reports_multiple_jsx_gaps_in_source_order() {
    let linter = linter_with(Box::new(NoMultiSpaces::default()));
    let source = r#"const A = () => <div  className="foo"  {...props}  id="bar"  data-x="y" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec![
            "vue/no-multi-spaces",
            "vue/no-multi-spaces",
            "vue/no-multi-spaces",
            "vue/no-multi-spaces"
        ]
    );
    assert_eq!(result.warning_count, 4);
    assert_eq!(result.error_count, 0);
    assert_eq!(
        diagnostic_slices(source, &result),
        vec!["  ", "  ", "  ", "  "]
    );
}

#[test]
fn migrated_no_multi_spaces_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(NoMultiSpaces::default()));
    let result = linter.lint_jsx(
        r#"const A = () => <div  className="foo" />;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated no-multi-spaces rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
}

#[test]
fn no_multi_spaces_sfc_offsets_template_diagnostics() {
    let source = r#"<script setup lang="ts">
const value = 1;
</script>

<template>
  <div  class="foo"></div>
</template>
"#;
    let linter = linter_with(Box::new(NoMultiSpaces::default()));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(diagnostic_rules(&result), vec!["vue/no-multi-spaces"]);
    assert_eq!(diagnostic_slices(source, &result), vec!["  "]);

    let expected = source.rfind("  class").unwrap() as u32;
    let diag = &result.diagnostics[0];
    assert_eq!(
        diag.start, expected,
        "SFC diagnostic must be offset into file coordinates"
    );
    assert_eq!(diag.end, expected + 2);
}

#[test]
fn no_multi_spaces_sfc_does_not_lint_script_tsx() {
    let source = r#"<script setup lang="tsx">
const A = () => <div  className="foo" />;
</script>
"#;
    let linter = linter_with(Box::new(NoMultiSpaces::default()));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        result.warning_count, 0,
        "SFC no-multi-spaces must not inspect JSX inside script blocks: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
}
