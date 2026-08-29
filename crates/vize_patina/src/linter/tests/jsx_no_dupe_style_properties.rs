use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::opinionated::html::NoDupeStyleProperties;
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
fn no_dupe_style_properties_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(NoDupeStyleProperties));
    let source = r#"const A = () => <div style="color: red; color: blue" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.warning_count, 1,
        "JSX duplicate style property must flag through the IR pass: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
    assert_eq!(
        diagnostic_rules(&result),
        vec!["html/no-dupe-style-properties"]
    );

    let diag = &result.diagnostics[0];
    let attr_start = source.find("style").unwrap() as u32;
    assert_eq!(
        diag.start, attr_start,
        "range must start at the written JSX style attribute"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        r#"style="color: red; color: blue""#
    );

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <div style="margin: 0; MARGIN: 1px" />;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["html/no-dupe-style-properties"],
        "TSX duplicate style property must also flag through the IR pass"
    );
}

#[test]
fn no_dupe_style_properties_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(NoDupeStyleProperties));
    for source in [
        r#"const A = () => <div style />;"#,
        r#"const A = () => <div style={{ color: a, color: b }} />;"#,
        r#"const A = () => <div style={'color:red;color:blue'} />;"#,
        r#"const A = () => <div {...props} />;"#,
        r#"const A = () => <div STYLE="color: red; color: blue" />;"#,
        r#"const A = () => <div html:style="color: red; color: blue" />;"#,
        r#"const A = () => <Component style="color: red; color: blue" />;"#,
        r#"const A = () => <Icons.Div style="color: red; color: blue" />;"#,
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
fn no_dupe_style_properties_reports_once_per_duplicate_not_per_backend() {
    let linter = linter_with(Box::new(NoDupeStyleProperties));
    let source =
        r#"const A = () => <div style="color: red; color: blue; margin: 0; margin: 1px" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        diagnostic_rules(&result),
        vec![
            "html/no-dupe-style-properties",
            "html/no-dupe-style-properties"
        ],
        "a migrated no-dupe-style-properties rule must not double-run: {:?}",
        result.diagnostics
    );

    let attr_start = source.find("style").unwrap() as u32;
    let attr_end =
        attr_start + r#"style="color: red; color: blue; margin: 0; margin: 1px""#.len() as u32;
    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.start, attr_start);
        assert_eq!(diagnostic.end, attr_end);
    }
}
