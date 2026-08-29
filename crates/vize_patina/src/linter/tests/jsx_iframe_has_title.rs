use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::IframeHasTitle;
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
fn iframe_has_title_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(IframeHasTitle));
    let source = r#"const A = () => <iframe src="https://example.com" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.warning_count, 1,
        "JSX iframe without title must flag through IR: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
    assert_eq!(diagnostic_rules(&result), vec!["a11y/iframe-has-title"]);

    let diag = &result.diagnostics[0];
    let element = r#"<iframe src="https://example.com" />"#;
    let iframe_start = source.find(element).unwrap() as u32;
    assert_eq!(
        diag.start, iframe_start,
        "range must start at the written JSX iframe"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        element,
        "range must cover the authored JSX iframe element"
    );
    assert_eq!(diag.severity, Severity::Warning);
    assert!(diag.help.is_some(), "diagnostic should keep rule help");

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <iframe src="https://example.com" />;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(tsx.warning_count, 1);
    assert_eq!(tsx.error_count, 0);
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/iframe-has-title"],
        "TSX iframe without title must also flag through IR"
    );
}

#[test]
fn iframe_has_title_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(IframeHasTitle));
    for source in [
        r#"const A = () => <iframe src="https://example.com" title="Example" />;"#,
        r#"const A = () => <iframe src="https://example.com" title="0" />;"#,
        r#"const A = () => <iframe src="https://example.com" title={frameTitle} />;"#,
        r#"const A = () => <iframe src="https://example.com" title={""} />;"#,
        r#"const A = () => <iframe src="https://example.com" title="" title="Example" />;"#,
        r#"const A = () => <iframe src="https://example.com" title="" title={frameTitle} />;"#,
        r#"const A = () => <Iframe src="https://example.com" />;"#,
        r#"const A = () => <Frame.iframe src="https://example.com" />;"#,
        r#"const A = () => <svg:iframe src="https://example.com" />;"#,
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
        r#"const A = () => <iframe src="https://example.com" />;"#,
        r#"const A = () => <iframe src="https://example.com" title />;"#,
        r#"const A = () => <iframe src="https://example.com" title="" />;"#,
        r#"const A = () => <iframe src="https://example.com" title="   " />;"#,
        r#"const A = () => <iframe src="https://example.com" TITLE="Example" />;"#,
        r#"const A = () => <iframe src="https://example.com" {...frameAttrs} />;"#,
        r#"const A = () => <iframe src="https://example.com" ns:title="Example" />;"#,
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
fn migrated_iframe_has_title_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(IframeHasTitle));
    let result = linter.lint_jsx(
        r#"const A = () => <iframe src="https://example.com" />;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated iframe-has-title rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
}
