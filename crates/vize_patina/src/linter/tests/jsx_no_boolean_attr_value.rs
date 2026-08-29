use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::vue::NoBooleanAttrValue;
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
fn no_boolean_attr_value_fires_on_jsx_and_tsx_lowered_markup() {
    let linter = linter_with(Box::new(NoBooleanAttrValue));
    let source = r#"const A = () => <input disabled="disabled" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["vue/no-boolean-attr-value"],
        "JSX explicit boolean attr value must flag through the lowered markup pass: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);

    let diag = &result.diagnostics[0];
    let attr = r#"disabled="disabled""#;
    let attr_start = source.find(attr).unwrap() as u32;
    assert_eq!(diag.start, attr_start);
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        attr,
        "range must cover the authored JSX attribute"
    );
    assert_eq!(diag.severity, Severity::Warning);
    assert_eq!(
        diag.help.as_deref(),
        Some(r#"Remove the value. Use just disabled instead of disabled="..."."#)
    );
    assert_eq!(diag.fix.as_ref().map(|_| "some"), None);

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <input disabled="disabled" />;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["vue/no-boolean-attr-value"],
        "TSX explicit boolean attr value must also flag through the lowered markup pass"
    );
}

#[test]
fn no_boolean_attr_value_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(NoBooleanAttrValue));
    for source in [
        r#"const A = () => <input disabled />;"#,
        r#"const A = () => <input disabled={isDisabled} />;"#,
        r#"const A = () => <input {...props} />;"#,
        r#"const A = () => <input type="text" />;"#,
        r#"const A = () => <input declare="declare" webkitdirectory="webkitdirectory" />;"#,
        r#"const A = () => <td nowrap="nowrap" />;"#,
        r#"const A = () => <my-button disabled="disabled" />;"#,
        r#"const A = () => <MyButton disabled="disabled" />;"#,
        r#"const A = () => <INPUT disabled="disabled" />;"#,
        r#"const A = () => <Forms.input disabled="disabled" />;"#,
        r#"const A = () => <input DISABLED="disabled" />;"#,
        r#"const A = () => <input autoFocus="autoFocus" />;"#,
        r#"const A = () => <svg:input disabled="disabled" />;"#,
        r#"const A = () => <svg:circle hidden="hidden" />;"#,
        r#"const A = () => <input html:disabled="disabled" />;"#,
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
        r#"const A = () => <input disabled="disabled" />;"#,
        r#"const A = () => <input disabled="" />;"#,
        r#"const A = () => <button disabled="true">Click</button>;"#,
        r#"const A = () => <svg hidden="hidden" />;"#,
        r#"const A = () => <div>{cond && <input disabled="disabled" />}</div>;"#,
        r#"const A = () => <ul>{items.map(() => <input disabled="disabled" />)}</ul>;"#,
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
fn no_boolean_attr_value_reports_multiple_attrs_in_source_order() {
    let linter = linter_with(Box::new(NoBooleanAttrValue));
    let source = r#"const A = () => <input disabled="disabled" required="required" checked />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["vue/no-boolean-attr-value", "vue/no-boolean-attr-value"]
    );
    assert_eq!(result.warning_count, 2);
    assert_eq!(result.error_count, 0);

    let slices: Vec<&str> = result
        .diagnostics
        .iter()
        .map(|diagnostic| &source[diagnostic.start as usize..diagnostic.end as usize])
        .collect();
    assert_eq!(
        slices,
        vec![r#"disabled="disabled""#, r#"required="required""#]
    );
    let fix_states: Vec<&str> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            if diagnostic.fix.is_some() {
                "some"
            } else {
                "none"
            }
        })
        .collect();
    assert_eq!(fix_states, vec!["none", "none"]);
}

#[test]
fn migrated_no_boolean_attr_value_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(NoBooleanAttrValue));
    let result = linter.lint_jsx(
        r#"const A = () => <input disabled="disabled" />;"#,
        "test.jsx",
        JsxLang::Jsx,
    );

    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated no-boolean-attr-value rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
}

#[test]
fn no_boolean_attr_value_sfc_offsets_template_diagnostics() {
    let source = r#"<script setup lang="ts">
const disabled = true;
</script>

<template>
  <input disabled="disabled" />
</template>
"#;
    let linter = linter_with(Box::new(NoBooleanAttrValue));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        diagnostic_rules(&result),
        vec!["vue/no-boolean-attr-value"],
        "SFC template boolean attr value must report once: {:?}",
        result.diagnostics
    );

    let diag = &result.diagnostics[0];
    let attr = r#"disabled="disabled""#;
    let attr_start = source.find(attr).unwrap() as u32;
    assert_eq!(
        diag.start, attr_start,
        "SFC diagnostic must be offset into file coordinates"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        attr,
        "range must cover the boolean attribute in the full SFC"
    );
}

#[test]
fn no_boolean_attr_value_sfc_does_not_lint_script_tsx() {
    let source = r#"<script setup lang="tsx">
const A = () => <input disabled="disabled" />;
</script>
"#;
    let linter = linter_with(Box::new(NoBooleanAttrValue));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        result.warning_count, 0,
        "SFC no-boolean-attr-value must not inspect JSX inside script blocks: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
}
