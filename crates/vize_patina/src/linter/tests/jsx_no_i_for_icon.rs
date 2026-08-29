use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::NoIForIcon;
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
fn no_i_for_icon_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(NoIForIcon));
    let source = "const A = () => <i class=\"fas fa-home\" />;";
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.warning_count, 1,
        "<i class=\"fas ...\"> must flag through the IR pass: {:?}",
        result.diagnostics
    );
    assert_eq!(diagnostic_rules(&result), vec!["a11y/no-i-for-icon"]);

    let diag = &result.diagnostics[0];
    let i_start = source.find("<i").unwrap() as u32;
    assert_eq!(diag.start, i_start, "range must start at the <i> tag");

    let tsx = linter.lint_jsx(
        "const A = (): JSX.Element => <i class=\"fas fa-home\" />;",
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/no-i-for-icon"],
        "TSX <i class=\"fas ...\"> must also flag through the IR pass"
    );
}

#[test]
fn no_i_for_icon_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(NoIForIcon));
    for source in [
        "const A = () => <i className=\"fas fa-home\" />;",
        "const A = () => <i class={iconClass} />;",
        "const A = () => <i class={'fas fa-home'} />;",
        "const A = () => <i {...props} />;",
        "const A = () => <Icons.i class=\"fas fa-home\" />;",
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
fn migrated_no_i_for_icon_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(NoIForIcon));
    let result = linter.lint_jsx(
        r#"const A = () => <i class="fas fa-home"/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated no-i-for-icon rule must report once: {:?}",
        result.diagnostics
    );
}
