use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::NoAriaHiddenOnFocusable;
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
fn no_aria_hidden_on_focusable_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(NoAriaHiddenOnFocusable));
    let source = r#"const A = () => <button aria-hidden="true" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.error_count, 1,
        "JSX focusable element with aria-hidden must flag through IR: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 0);
    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/no-aria-hidden-on-focusable"]
    );

    let diag = &result.diagnostics[0];
    let element = r#"<button aria-hidden="true" />"#;
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
        r#"const A = (): JSX.Element => <a href="/" aria-hidden="true">Home</a>;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/no-aria-hidden-on-focusable"],
        "TSX anchors with href must also flag through IR"
    );
}

#[test]
fn no_aria_hidden_on_focusable_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(NoAriaHiddenOnFocusable));
    for source in [
        r#"const A = () => <div aria-hidden="true" />;"#,
        r#"const A = () => <a aria-hidden="true">decorative</a>;"#,
        r#"const A = () => <button aria-hidden="false" />;"#,
        r#"const A = () => <button aria-hidden={true} />;"#,
        r#"const A = () => <button aria-hidden />;"#,
        r#"const A = () => <button aria-hidden aria-hidden="true" />;"#,
        r#"const A = () => <button ARIA-HIDDEN="true" />;"#,
        r#"const A = () => <button ariaHidden="true" />;"#,
        r#"const A = () => <button a11y:aria-hidden="true" />;"#,
        r#"const A = () => <a {...props} aria-hidden="true">Home</a>;"#,
        r#"const A = () => <div tabIndex="0" aria-hidden="true" />;"#,
        r#"const A = () => <div contentEditable="true" aria-hidden="true" />;"#,
        r#"const A = () => <Button aria-hidden="true" />;"#,
        r#"const A = () => <Forms.button aria-hidden="true" />;"#,
        r#"const A = () => <ui:button aria-hidden="true" />;"#,
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
        r#"const A = () => <button aria-hidden="true" />;"#,
        r#"const A = () => <input aria-hidden="true" />;"#,
        r#"const A = () => <area href="/map" aria-hidden="true" />;"#,
        r#"const A = () => <a href="/" aria-hidden="true">Home</a>;"#,
        r#"const A = () => <a href={url} aria-hidden="true">Home</a>;"#,
        r#"const A = () => <div tabindex="0" aria-hidden="true" />;"#,
        r#"const A = () => <div tabindex="x" aria-hidden="true" />;"#,
        r#"const A = () => <div contenteditable="true" aria-hidden="true" />;"#,
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
fn migrated_no_aria_hidden_on_focusable_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(NoAriaHiddenOnFocusable));
    let result = linter.lint_jsx(
        r#"const A = () => <button aria-hidden="true" />;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated no-aria-hidden-on-focusable rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);
}
