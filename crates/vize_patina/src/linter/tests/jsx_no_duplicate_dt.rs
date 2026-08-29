use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::html::NoDuplicateDt;
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
fn no_duplicate_dt_runs_over_lowered_markup_ir_once() {
    let linter = linter_with(Box::new(NoDuplicateDt));
    let source = r#"const A = () => <dl><dt>A</dt><dd>def 1</dd><dt>A</dt></dl>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["html/no-duplicate-dt"],
        "migrated no-duplicate-dt must report once via lowered markup IR: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);

    let diag = &result.diagnostics[0];
    let duplicate_start = source.rfind("<dt>").unwrap() as u32;
    assert_eq!(diag.severity, Severity::Warning);
    assert!(diag.help.is_some(), "diagnostic should carry help text");
    assert!(diag.fix.is_none(), "no-duplicate-dt is not auto-fixable");
    assert!(
        diag.message.contains("A"),
        "diagnostic message should name the duplicate term: {diag:?}"
    );
    assert_eq!(diag.start, duplicate_start);
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        "<dt>A</dt>",
        "range must cover the duplicate authored JSX <dt>"
    );

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <dl><dt>A</dt><dt>A</dt></dl>;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["html/no-duplicate-dt"],
        "TSX keeps the same lowered markup IR behavior"
    );
}

#[test]
fn no_duplicate_dt_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(NoDuplicateDt));
    for source in [
        r#"const A = () => <dl><dt>A</dt><dt>B</dt></dl>;"#,
        r#"const A = () => <div><dt>A</dt><dt>A</dt></div>;"#,
        r#"const A = () => <dl><div><dt>A</dt></div><dt>A</dt></dl>;"#,
        r#"const A = () => <dl>{cond && <dt>A</dt>}<dt>A</dt></dl>;"#,
        r#"const A = () => <dl>{items.map(() => <dt>A</dt>)}<dt>A</dt></dl>;"#,
        r#"const A = () => <dl><dt>A{'!'}</dt><dt>A</dt></dl>;"#,
        r#"const A = () => <Dl><dt>A</dt><dt>A</dt></Dl>;"#,
        r#"const A = () => <DL><dt>A</dt><dt>A</dt></DL>;"#,
        r#"const A = () => <dl><DT>A</DT><dt>A</dt></dl>;"#,
        r#"const A = () => <svg:dl><dt>A</dt><dt>A</dt></svg:dl>;"#,
        r#"const A = () => <dl><svg:dt>A</svg:dt><dt>A</dt></dl>;"#,
        r#"const A = () => <Lists.dl><dt>A</dt><dt>A</dt></Lists.dl>;"#,
        r#"const A = () => <dl><Terms.dt>A</Terms.dt><dt>A</dt></dl>;"#,
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
fn no_duplicate_dt_reports_repeated_terms_without_fallback_double_run() {
    let linter = linter_with(Box::new(NoDuplicateDt));
    let source = r#"const A = () => <dl><dt>X</dt><dt>X</dt><dt>X</dt></dl>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["html/no-duplicate-dt", "html/no-duplicate-dt"],
        "each repeated term after the first reports once: {:?}",
        result.diagnostics
    );

    let expected_starts: Vec<u32> = source
        .match_indices("<dt>X</dt>")
        .skip(1)
        .map(|(offset, _)| offset as u32)
        .collect();
    let actual_starts: Vec<u32> = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.start)
        .collect();
    assert_eq!(actual_starts, expected_starts);

    for diagnostic in &result.diagnostics {
        assert_eq!(
            &source[diagnostic.start as usize..diagnostic.end as usize],
            "<dt>X</dt>"
        );
    }
}

#[test]
fn no_duplicate_dt_keeps_lowered_jsx_fragment_text_behavior() {
    let linter = linter_with(Box::new(NoDuplicateDt));

    for source in [
        r#"const A = () => <dl><dt>{'A'}</dt><dt>A</dt></dl>;"#,
        r#"const A = () => <dl><><dt>A</dt><dt>A</dt></></dl>;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            diagnostic_rules(&result),
            vec!["html/no-duplicate-dt"],
            "lowered JSX fallback boundary changed for {source}: {:?}",
            result.diagnostics
        );
    }
}
