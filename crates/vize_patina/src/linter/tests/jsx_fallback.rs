//! Cross-application coverage for unmigrated rules that fire on JSX/TSX via
//! the lowering fallback path in [`Linter::lint_jsx`].
//!
//! Each rule here lacks a [`MarkupRule`](crate::markup::MarkupRule) projection
//! today, so the JSX path must still drive them over the relief AST produced
//! by `vize_atelier_jsx::lower_source`. The IR-pass tests live in the sibling
//! `jsx` module.

use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::vue::{HtmlSelfClosing, NoDuplicateAttributes, NoUnsafeUrl};
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
fn fallback_no_duplicate_attributes_fires_on_jsx() {
    let linter = linter_with(Box::new(NoDuplicateAttributes::default()));
    let result = linter.lint_jsx(
        r#"const A = () => <div id="a" id="b"/>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.error_count, 1,
        "duplicate JSX props must flag via fallback: {:?}",
        result.diagnostics
    );
    assert_eq!(
        diagnostic_rules(&result),
        vec!["vue/no-duplicate-attributes"]
    );
}

#[test]
fn fallback_html_self_closing_fires_on_jsx() {
    let linter = linter_with(Box::new(HtmlSelfClosing::default()));
    let result = linter.lint_jsx(
        "const A = () => <MyWidget></MyWidget>;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.warning_count, 1,
        "empty JSX components written with paired tags must flag via fallback: {:?}",
        result.diagnostics
    );
    assert_eq!(diagnostic_rules(&result), vec!["vue/html-self-closing"]);
}

#[test]
fn fallback_no_unsafe_url_fires_on_jsx() {
    let linter = linter_with(Box::new(NoUnsafeUrl));
    let result = linter.lint_jsx(
        r#"const A = () => <a href="javascript:alert(1)">x</a>;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.warning_count, 1,
        "unsafe static JSX URLs must flag via fallback: {:?}",
        result.diagnostics
    );
    assert_eq!(diagnostic_rules(&result), vec!["vue/no-unsafe-url"]);
}
