use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::html::RequireDatetime;
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
fn require_datetime_runs_over_lowered_markup_ir_once() {
    let linter = linter_with(Box::new(RequireDatetime));
    let source = r#"const A = () => <time>Christmas</time>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["html/require-datetime"],
        "migrated require-datetime must report once via lowered markup IR: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);

    let diag = &result.diagnostics[0];
    let start = source.find("<time>").unwrap() as u32;
    assert_eq!(diag.severity, Severity::Warning);
    assert!(diag.help.is_some(), "diagnostic should carry help text");
    assert!(diag.fix.is_none(), "require-datetime is not auto-fixable");
    assert_eq!(diag.start, start);
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        "<time>Christmas</time>",
        "range must cover the authored JSX <time>"
    );

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <time>Christmas</time>;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["html/require-datetime"],
        "TSX keeps the same lowered markup IR behavior"
    );
}

#[test]
fn require_datetime_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(RequireDatetime));
    for source in [
        r#"const A = () => <time datetime="2024-12-25">Christmas</time>;"#,
        r#"const A = () => <time datetime={date}>Christmas</time>;"#,
        r#"const A = () => <time v-bind:datetime={date}>Christmas</time>;"#,
        r#"const A = () => <time>2024-12-25</time>;"#,
        r#"const A = () => <time><>{'2024-12-25'}</></time>;"#,
        r#"const A = () => <time>{formattedDate}</time>;"#,
        r#"const A = () => <time>last {unit}</time>;"#,
        r#"const A = () => <Time>Christmas</Time>;"#,
        r#"const A = () => <Foo.time>Christmas</Foo.time>;"#,
        r#"const A = () => <time-clock>Christmas</time-clock>;"#,
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
fn require_datetime_reports_each_invalid_time_without_fallback_double_run() {
    let linter = linter_with(Box::new(RequireDatetime));
    let source =
        r#"const A = () => <><time {...attrs}>Christmas</time><time>last Tuesday</time></>;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["html/require-datetime", "html/require-datetime"],
        "each invalid time reports once through the migrated lane: {:?}",
        result.diagnostics
    );

    let ranges: Vec<&str> = result
        .diagnostics
        .iter()
        .map(|diagnostic| &source[diagnostic.start as usize..diagnostic.end as usize])
        .collect();
    assert_eq!(
        ranges,
        vec![
            "<time {...attrs}>Christmas</time>",
            "<time>last Tuesday</time>"
        ],
        "diagnostics should preserve source order and authored element ranges"
    );

    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert!(diagnostic.help.is_some());
        assert!(diagnostic.fix.is_none());
    }
}
