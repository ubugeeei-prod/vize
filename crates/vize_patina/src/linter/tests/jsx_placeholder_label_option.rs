use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::PlaceholderLabelOption;
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

#[test]
fn placeholder_label_option_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(PlaceholderLabelOption));
    let source = r#"const A = () => <select required><option value="a">Choose</option></select>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/placeholder-label-option"],
        "JSX placeholder option must flag through the IR pass: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);

    let diag = &result.diagnostics[0];
    let option = r#"<option value="a">Choose</option>"#;
    let option_start = source.find(option).unwrap() as u32;
    assert_eq!(diag.start, option_start);
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        option,
        "range must cover the authored JSX option element"
    );
    assert_eq!(diag.severity, Severity::Warning);
    assert!(diag.help.is_some(), "diagnostic should keep rule help");

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <select required><option value="a">Choose</option></select>;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/placeholder-label-option"],
        "TSX placeholder option must also flag through the IR pass"
    );
}

#[test]
fn placeholder_label_option_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(PlaceholderLabelOption));
    for source in [
        r#"const A = () => <select required><option value="" disabled>Choose</option></select>;"#,
        r#"const A = () => <select required><option value="" hidden>Choose</option></select>;"#,
        r#"const A = () => <select><option>Choose</option></select>;"#,
        r#"const A = () => <select required><option value={""}>Choose</option></select>;"#,
        r#"const A = () => <Select required><option value="a">Choose</option></Select>;"#,
        r#"const A = () => <Forms.select required><option value="a">Choose</option></Forms.select>;"#,
        r#"const A = () => <svg:select required><option value="a">Choose</option></svg:select>;"#,
        r#"const A = () => <select required><Option value="a">Choose</Option></select>;"#,
        r#"const A = () => <select required><svg:option value="a">Choose</svg:option></select>;"#,
        r#"const A = () => <select required><span><option value="a">Nested</option></span></select>;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 0,
            "must stay clean for {source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.error_count, 0, "must not error for {source}");
    }

    for source in [
        r#"const A = () => <select required><option>Choose</option></select>;"#,
        r#"const A = () => <select required><option value="a">Choose</option></select>;"#,
        r#"const A = () => <select required><option VALUE="">Choose</option></select>;"#,
        r#"const A = () => <select required><optgroup label="G"><option value="">Choose</option></optgroup></select>;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 1,
            "must keep warning for {source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.error_count, 0, "must not error for {source}");
    }
}

#[test]
fn migrated_placeholder_label_option_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(PlaceholderLabelOption));
    let result = linter.lint_jsx(
        r#"const A = () => <select required><option value="a">Choose</option></select>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated placeholder-label-option rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
}

#[test]
fn placeholder_label_option_sfc_offsets_template_diagnostics() {
    let source = r#"<script setup lang="ts">
const choices = ["A"];
</script>

<template>
  <select required>
    <option value="b">Choose</option>
    <option value="a">A</option>
  </select>
</template>
"#;
    let linter = linter_with(Box::new(PlaceholderLabelOption));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/placeholder-label-option"],
        "SFC template placeholder option must report once: {:?}",
        result.diagnostics
    );

    let diag = &result.diagnostics[0];
    let option = r#"<option value="b">"#;
    let option_start = source.find(option).unwrap() as u32;
    assert_eq!(
        diag.start, option_start,
        "SFC diagnostic must be offset into file coordinates"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        option,
        "range must cover the placeholder option in the full SFC"
    );
}

#[test]
fn placeholder_label_option_sfc_does_not_lint_script_tsx() {
    let source = r#"<script setup lang="tsx">
const A = () => <select required><option value="a">Choose</option></select>;
</script>
"#;
    let linter = linter_with(Box::new(PlaceholderLabelOption));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        result.warning_count, 0,
        "SFC placeholder-label-option must not inspect JSX inside script blocks: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
}
