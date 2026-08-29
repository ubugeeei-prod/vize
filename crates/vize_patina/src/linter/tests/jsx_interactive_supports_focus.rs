use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::InteractiveSupportsFocus;
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
fn interactive_supports_focus_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(InteractiveSupportsFocus));
    let source = r#"const A = () => <div role="button" onClick={handle}>Click</div>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.warning_count, 1,
        "JSX element with interactive role must flag through IR: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/interactive-supports-focus"]
    );

    let diag = &result.diagnostics[0];
    let element = r#"<div role="button" onClick={handle}>Click</div>"#;
    let element_start = source.find(element).unwrap() as u32;
    assert_eq!(
        diag.start, element_start,
        "range must start at the written JSX element"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        element,
        "range must cover the authored JSX element"
    );
    assert_eq!(diag.severity, Severity::Warning);
    assert!(diag.help.is_some(), "diagnostic should keep rule help");

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <span role="link">Home</span>;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(tsx.warning_count, 1);
    assert_eq!(tsx.error_count, 0);
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/interactive-supports-focus"],
        "TSX spans with interactive roles must also flag through IR"
    );
}

#[test]
fn interactive_supports_focus_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(InteractiveSupportsFocus));
    for source in [
        r#"const A = () => <div role="presentation" />;"#,
        r#"const A = () => <div role="Button" />;"#,
        r#"const A = () => <div role="button " />;"#,
        r#"const A = () => <div role="button link" />;"#,
        r#"const A = () => <div ROLE="button" />;"#,
        r#"const A = () => <div role={role} />;"#,
        r#"const A = () => <div role={"button"} />;"#,
        r#"const A = () => <div role />;"#,
        r#"const A = () => <div role role="button" />;"#,
        r#"const A = () => <div role="presentation" role="button" />;"#,
        r#"const A = () => <div role="button" tabindex="0" />;"#,
        r#"const A = () => <div role="button" tabindex="" />;"#,
        r#"const A = () => <div role="button" tabindex="x" />;"#,
        r#"const A = () => <div role="button" contenteditable="true" />;"#,
        r#"const A = () => <div role="button" contenteditable="" />;"#,
        r#"const A = () => <div role="button" contenteditable="plaintext-only" />;"#,
        r#"const A = () => <button role="link" />;"#,
        r#"const A = () => <a role="button">Decorative link</a>;"#,
        r#"const A = () => <details role="button" />;"#,
        r#"const A = () => <audio role="button" />;"#,
        r#"const A = () => <video role="button" />;"#,
        r#"const A = () => <area href="/map" role="button" />;"#,
        r#"const A = () => <area href={map} role="button" />;"#,
        r#"const A = () => <Button role="button" />;"#,
        r#"const A = () => <Forms.button role="button" />;"#,
        r#"const A = () => <div a11y:role="button" />;"#,
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
        r#"const A = () => <div role="button" />;"#,
        r#"const A = () => <span role="link" />;"#,
        r#"const A = () => <div role="button" role="presentation" />;"#,
        r#"const A = () => <div role="button" tabindex="-1" />;"#,
        r#"const A = () => <div role="button" tabIndex="0" />;"#,
        r#"const A = () => <div role="button" contentEditable="true" />;"#,
        r#"const A = () => <area role="button" />;"#,
        r#"const A = () => <area ns:href={map} role="button" />;"#,
        r#"const A = () => <ui:button role="button" />;"#,
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
fn migrated_interactive_supports_focus_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(InteractiveSupportsFocus));
    let result = linter.lint_jsx(
        r#"const A = () => <div role="button" onClick={handle}>Click</div>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated interactive-supports-focus rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
}
