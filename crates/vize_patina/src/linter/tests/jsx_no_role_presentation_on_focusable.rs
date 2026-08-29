use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::NoRolePresentationOnFocusable;
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
fn no_role_presentation_on_focusable_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(NoRolePresentationOnFocusable));
    let source = r#"const A = () => <button role="presentation" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.error_count, 1,
        "JSX focusable element with presentation role must flag through IR: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 0);
    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/no-role-presentation-on-focusable"]
    );

    let diag = &result.diagnostics[0];
    let element = r#"<button role="presentation" />"#;
    let button_start = source.find(element).unwrap() as u32;
    assert_eq!(
        diag.start, button_start,
        "range must start at the written JSX element"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        element,
        "range must cover the authored JSX element"
    );
    assert_eq!(diag.severity, Severity::Error);
    assert!(diag.help.is_some(), "diagnostic should keep rule help");

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <a href="/" role="presentation">Home</a>;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(tsx.error_count, 1);
    assert_eq!(tsx.warning_count, 0);
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/no-role-presentation-on-focusable"],
        "TSX anchors with href must also flag through IR"
    );
}

#[test]
fn no_role_presentation_on_focusable_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(NoRolePresentationOnFocusable));
    for source in [
        r#"const A = () => <div role="presentation" />;"#,
        r#"const A = () => <a role="presentation">decorative</a>;"#,
        r#"const A = () => <button role="button" />;"#,
        r#"const A = () => <button role="Presentation" />;"#,
        r#"const A = () => <button role="presentation " />;"#,
        r#"const A = () => <button role="presentation button" />;"#,
        r#"const A = () => <button ROLE="presentation" />;"#,
        r#"const A = () => <button role={role} />;"#,
        r#"const A = () => <button role={"presentation"} />;"#,
        r#"const A = () => <button role />;"#,
        r#"const A = () => <button role role="presentation" />;"#,
        r#"const A = () => <button role="button" role="presentation" />;"#,
        r#"const A = () => <a {...props} role="presentation">Home</a>;"#,
        r#"const A = () => <div tabIndex="0" role="presentation" />;"#,
        r#"const A = () => <div contentEditable="true" role="presentation" />;"#,
        r#"const A = () => <Button role="presentation" />;"#,
        r#"const A = () => <Forms.button role="presentation" />;"#,
        r#"const A = () => <ui:button role="presentation" />;"#,
        r#"const A = () => <button a11y:role="presentation" />;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.error_count, 0,
            "must stay clean for {source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.warning_count, 0, "must not warn for {source}");
    }

    for source in [
        r#"const A = () => <button role="presentation" />;"#,
        r#"const A = () => <button role="none" />;"#,
        r#"const A = () => <button role="presentation" role="button" />;"#,
        r#"const A = () => <input role="none" />;"#,
        r#"const A = () => <area href="/map" role="presentation" />;"#,
        r#"const A = () => <a href="/" role="presentation">Home</a>;"#,
        r#"const A = () => <a href={url} role="presentation">Home</a>;"#,
        r#"const A = () => <div tabindex="0" role="presentation" />;"#,
        r#"const A = () => <div tabindex="x" role="presentation" />;"#,
        r#"const A = () => <div tabindex="" role="presentation" />;"#,
        r#"const A = () => <div contenteditable="true" role="presentation" />;"#,
        r#"const A = () => <div contenteditable="" role="presentation" />;"#,
        r#"const A = () => <div contenteditable="plaintext-only" role="presentation" />;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.error_count, 1,
            "must keep error for {source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.warning_count, 0, "must not warn for {source}");
    }
}

#[test]
fn migrated_no_role_presentation_on_focusable_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(NoRolePresentationOnFocusable));
    let result = linter.lint_jsx(
        r#"const A = () => <button role="presentation" />;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated no-role-presentation-on-focusable rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);
}
