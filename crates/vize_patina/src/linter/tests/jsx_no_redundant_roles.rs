use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::NoRedundantRoles;
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
fn no_redundant_roles_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(NoRedundantRoles));
    let source = r#"const A = () => <nav role="navigation" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.warning_count, 1,
        "JSX nav/navigation must flag through the IR pass: {:?}",
        result.diagnostics
    );
    assert_eq!(diagnostic_rules(&result), vec!["a11y/no-redundant-roles"]);

    let diag = &result.diagnostics[0];
    let nav_start = source.find("<nav").unwrap() as u32;
    assert_eq!(diag.start, nav_start, "range must start at the <nav> tag");

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <button role="button" />;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/no-redundant-roles"],
        "TSX button/button must also flag through the IR pass"
    );
}

#[test]
fn no_redundant_roles_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(NoRedundantRoles));
    for source in [
        r#"const A = () => <div role="navigation" />;"#,
        r#"const A = () => <nav role={role} />;"#,
        r#"const A = () => <button role={"button"} />;"#,
        r#"const A = () => <button role />;"#,
        r#"const A = () => <button Role="button" />;"#,
        r#"const A = () => <button role="Button" />;"#,
        r#"const A = () => <button role="button " />;"#,
        r#"const A = () => <button role="button checkbox" />;"#,
        r#"const A = () => <a href={url} role="link" />;"#,
        r#"const A = () => <Nav role="navigation" />;"#,
        r#"const A = () => <input type={type} role="checkbox" />;"#,
        r#"const A = () => <svg:button role="button" />;"#,
        r#"const A = () => <Forms.button role="button" />;"#,
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
fn migrated_no_redundant_roles_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(NoRedundantRoles));
    let result = linter.lint_jsx(
        r#"const A = () => <main role="main" />;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated no-redundant-roles rule must report once: {:?}",
        result.diagnostics
    );
}
