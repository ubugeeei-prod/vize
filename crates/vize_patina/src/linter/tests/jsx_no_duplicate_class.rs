use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::opinionated::html::NoDuplicateClass;
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
fn no_duplicate_class_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(NoDuplicateClass));
    let source = r#"const A = () => <div class="btn btn primary" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.warning_count, 1,
        "JSX duplicate class must flag through the IR pass: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
    assert_eq!(diagnostic_rules(&result), vec!["html/no-duplicate-class"]);

    let diag = &result.diagnostics[0];
    let attr_start = source.find("class").unwrap() as u32;
    assert_eq!(
        diag.start, attr_start,
        "range must start at the written JSX class attribute"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        r#"class="btn btn primary""#
    );

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <div class="btn btn" />;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["html/no-duplicate-class"],
        "TSX duplicate class must also flag through the IR pass"
    );
}

#[test]
fn no_duplicate_class_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(NoDuplicateClass));
    for source in [
        r#"const A = () => <div className="btn btn" />;"#,
        r#"const A = () => <div class />;"#,
        r#"const A = () => <div class={'btn btn'} />;"#,
        r#"const A = () => <div class={classes} />;"#,
        r#"const A = () => <div {...props} />;"#,
        r#"const A = () => <div CLASS="btn btn" />;"#,
        r#"const A = () => <div html:class="btn btn" />;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 0,
            "must stay clean for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn no_duplicate_class_reports_once_per_duplicate_token_not_per_backend() {
    let linter = linter_with(Box::new(NoDuplicateClass));
    let source = r#"const A = () => <div class="a a b b" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        diagnostic_rules(&result),
        vec!["html/no-duplicate-class", "html/no-duplicate-class"],
        "a migrated no-duplicate-class rule must not double-run: {:?}",
        result.diagnostics
    );

    let attr_start = source.find("class").unwrap() as u32;
    let attr_end = attr_start + r#"class="a a b b""#.len() as u32;
    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.start, attr_start);
        assert_eq!(diagnostic.end, attr_end);
    }
}
