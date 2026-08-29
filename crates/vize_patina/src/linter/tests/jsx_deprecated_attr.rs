use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::html::DeprecatedAttr;
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
fn deprecated_attr_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(DeprecatedAttr));
    let source = r#"const A = () => <div align="center" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.warning_count, 1,
        "JSX align attr must flag through the IR pass: {:?}",
        result.diagnostics
    );
    assert_eq!(diagnostic_rules(&result), vec!["html/deprecated-attr"]);

    let diag = &result.diagnostics[0];
    let attr_start = source.find("align").unwrap() as u32;
    assert_eq!(
        diag.start, attr_start,
        "range must start at the written JSX attribute"
    );

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <table cellpadding="5" />;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["html/deprecated-attr"],
        "TSX lowercase cellpadding must also flag through the IR pass"
    );
}

#[test]
fn deprecated_attr_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(DeprecatedAttr));
    for source in [
        r#"const A = () => <table border="1" />;"#,
        r#"const A = () => <table cellPadding="5" />;"#,
        r#"const A = () => <table cellpadding={5} />;"#,
        r#"const A = () => <div ALIGN="center" />;"#,
        r#"const A = () => <Table align="center" />;"#,
        r#"const A = () => <div html:align="center" />;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 0,
            "must stay clean for {source}: {:?}",
            result.diagnostics
        );
    }

    let namespaced_table = r#"const A = () => <svg:table border="1" />;"#;
    let result = linter.lint_jsx(namespaced_table, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.warning_count, 1,
        "namespaced tags must keep the old lowered-tag table exception boundary"
    );
}

#[test]
fn migrated_deprecated_attr_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(DeprecatedAttr));
    let result = linter.lint_jsx(
        r#"const A = () => <body background="/bg.png" />;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated deprecated-attr rule must report once: {:?}",
        result.diagnostics
    );
}
