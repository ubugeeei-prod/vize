use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::HeadingHasContent;
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
fn heading_has_content_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(HeadingHasContent));
    let source = r#"const A = () => <h1 />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/heading-has-content"],
        "JSX heading without content must flag through IR: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);

    let diag = &result.diagnostics[0];
    let element = "<h1 />";
    let h1_start = source.find(element).unwrap() as u32;
    assert_eq!(diag.start, h1_start);
    assert_eq!(&source[diag.start as usize..diag.end as usize], element);
    assert_eq!(diag.severity, Severity::Warning);
    assert!(diag.help.is_some(), "diagnostic should keep rule help");

    let tsx = linter.lint_jsx(
        "const A = (): JSX.Element => <h1 />;",
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/heading-has-content"],
        "TSX heading without content must also flag through IR"
    );
    assert_eq!(tsx.warning_count, 1);
    assert_eq!(tsx.error_count, 0);
}

#[test]
fn heading_has_content_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(HeadingHasContent));
    for source in [
        r#"const A = () => <h1>Title</h1>;"#,
        r#"const A = () => <h1>{title}</h1>;"#,
        r#"const A = () => <h1>{0}</h1>;"#,
        r#"const A = () => <h1>{'Title'}</h1>;"#,
        r#"const A = () => <h1><span>Title</span></h1>;"#,
        r#"const A = () => <h1><slot /></h1>;"#,
        r#"const A = () => <h1><><slot /></></h1>;"#,
        r#"const A = () => <h1 aria-hidden="true" />;"#,
        r#"const A = () => <h1 aria-label="" />;"#,
        r#"const A = () => <h1 aria-label={title} />;"#,
        r#"const A = () => <h1 aria-labelledby={labelId} />;"#,
        r#"const A = () => <H1 />;"#,
        r#"const A = () => <Headings.h1 />;"#,
        r#"const A = () => <svg:h1 />;"#,
        r#"const A = () => <Comp render={<h1 />} />;"#,
        r#"const A = () => <Comp render={() => <h1 />} />;"#,
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
        r#"const A = () => <h1 />;"#,
        r#"const A = () => <h1>{}</h1>;"#,
        r#"const A = () => <h1>{''}</h1>;"#,
        r#"const A = () => <h1>{' '}</h1>;"#,
        r#"const A = () => <h1>{ok && <span>Title</span>}</h1>;"#,
        r#"const A = () => <h1>{ok ? <span>Title</span> : null}</h1>;"#,
        r#"const A = () => <h1>{items.map((item) => <span>{item}</span>)}</h1>;"#,
        r#"const A = () => <h1 aria-hidden />;"#,
        r#"const A = () => <h1 aria-hidden="True" />;"#,
        r#"const A = () => <h1 aria-hidden={true} />;"#,
        r#"const A = () => <h1 ARIA-LABEL="Title" />;"#,
        r#"const A = () => <h1 a11y:aria-label="Title" />;"#,
        r#"const A = () => <h1 {...attrs} />;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            diagnostic_rules(&result),
            vec!["a11y/heading-has-content"],
            "must keep one warning for {source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.error_count, 0, "must not error for {source}");
    }
}

#[test]
fn migrated_heading_has_content_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(HeadingHasContent));
    let result = linter.lint_jsx(r#"const A = () => <h1 />;"#, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated heading-has-content rule must report once: {:?}",
        result.diagnostics
    );
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
}
