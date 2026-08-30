use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::vue::NoInlineStyle;
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
fn no_inline_style_fires_on_jsx_and_tsx_markup() {
    let linter = linter_with(Box::new(NoInlineStyle));
    let source = r#"const A = () => <div style="color:red" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(diagnostic_rules(&result), vec!["vue/no-inline-style"]);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
    assert_eq!(
        diagnostic_slices(source, &result),
        vec![r#"style="color:red""#]
    );

    let diag = &result.diagnostics[0];
    assert_eq!(diag.severity, Severity::Warning);
    assert_eq!(diag.message, "Avoid using inline style attributes");
    assert_eq!(
        diag.help.as_deref(),
        Some("Use CSS classes or scoped styles instead")
    );
    assert!(diag.fix.is_none());

    let tsx = r#"const A = (): JSX.Element => <div style={{ color: activeColor }} />;"#;
    let tsx_result = linter.lint_jsx(tsx, "test.tsx", JsxLang::Tsx);
    assert_eq!(
        diagnostic_slices(tsx, &tsx_result),
        vec![r#"style={{ color: activeColor }}"#],
        "TSX should use the same authored source range"
    );
}

#[test]
fn no_inline_style_preserves_jsx_clean_boundaries() {
    let linter = linter_with(Box::new(NoInlineStyle));
    for source in [
        r#"const A = () => <div className="foo" />;"#,
        r#"const A = () => <div {...props} />;"#,
        r#"const A = () => <div STYLE="color:red" />;"#,
        r#"const A = () => <div foo:style="x" />;"#,
        r#"const A = () => <div v-bind:Style={styles} />;"#,
        r#"const A = () => <div onStyle={handler} />;"#,
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
fn no_inline_style_reports_multiple_jsx_styles_in_source_order() {
    let linter = linter_with(Box::new(NoInlineStyle));
    let source = r#"const A = () => <><div style="color:red" /><span style={{ marginTop }} /></>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["vue/no-inline-style", "vue/no-inline-style"]
    );
    assert_eq!(result.warning_count, 2);
    assert_eq!(result.error_count, 0);
    assert_eq!(
        diagnostic_slices(source, &result),
        vec![r#"style="color:red""#, r#"style={{ marginTop }}"#]
    );
}

#[test]
fn no_inline_style_reports_nested_jsx_styles() {
    let linter = linter_with(Box::new(NoInlineStyle));
    let source = r#"const A = () => <div>{cond && <span style={dynamicStyles} />}</div>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(diagnostic_rules(&result), vec!["vue/no-inline-style"]);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
    assert_eq!(
        diagnostic_slices(source, &result),
        vec![r#"style={dynamicStyles}"#]
    );
}

#[test]
fn no_inline_style_supports_jsx_v_bind_style_spelling() {
    let linter = linter_with(Box::new(NoInlineStyle));
    let source = r#"const A = () => <div v-bind:style={styles} />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(diagnostic_rules(&result), vec!["vue/no-inline-style"]);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
    assert_eq!(
        diagnostic_slices(source, &result),
        vec![r#"v-bind:style={styles}"#]
    );
}

#[test]
fn migrated_no_inline_style_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(NoInlineStyle));
    let result = linter.lint_jsx(
        r#"const A = () => <div style="color:red" />;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated no-inline-style rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
}

#[test]
fn no_inline_style_sfc_offsets_template_diagnostics() {
    let source = r#"<script setup lang="ts">
const value = 1;
</script>

<template>
  <div style="color:red"></div>
</template>
"#;
    let linter = linter_with(Box::new(NoInlineStyle));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(diagnostic_rules(&result), vec!["vue/no-inline-style"]);
    assert_eq!(
        diagnostic_slices(source, &result),
        vec![r#"style="color:red""#]
    );

    let expected = source.rfind(r#"style="color:red""#).unwrap() as u32;
    let diag = &result.diagnostics[0];
    assert_eq!(
        diag.start, expected,
        "SFC diagnostic must be offset into file coordinates"
    );
    assert_eq!(diag.end, expected + r#"style="color:red""#.len() as u32);
}

#[test]
fn no_inline_style_sfc_does_not_lint_script_tsx() {
    let source = r#"<script setup lang="tsx">
const A = () => <div style="color:red" />;
</script>
"#;
    let linter = linter_with(Box::new(NoInlineStyle));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        result.warning_count, 0,
        "SFC no-inline-style must not inspect JSX inside script blocks: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
}
