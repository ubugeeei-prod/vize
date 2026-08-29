use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::html::NoConsecutiveBr;
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
fn no_consecutive_br_runs_over_lowered_markup_ir_once() {
    let source = "const A = () => <p>line<br /><br />more</p>;";
    let linter = linter_with(Box::new(NoConsecutiveBr));
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["html/no-consecutive-br"],
        "migrated no-consecutive-br must report once via lowered markup IR: {:?}",
        result.diagnostics
    );

    let diag = &result.diagnostics[0];
    let second_br = source.rfind("<br").unwrap() as u32;
    assert_eq!(
        diag.start, second_br,
        "range must start at the second JSX <br>"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        "<br />",
        "range must cover exactly the authored JSX <br />"
    );

    let tsx = linter.lint_jsx(
        "const A = (): JSX.Element => <p>line<br /><br />more</p>;",
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["html/no-consecutive-br"],
        "TSX keeps the same lowered markup IR behavior"
    );
}

#[test]
fn no_consecutive_br_preserves_lowered_jsx_boundaries() {
    let linter = linter_with(Box::new(NoConsecutiveBr));
    for source in [
        "const A = () => <p>line<br />line</p>;",
        "const A = () => <p>line<br />{'x'}<br />more</p>;",
        "const A = () => <p>line<br />{spacer}<br />more</p>;",
        "const A = () => <p><br />{cond && <br />}<br /></p>;",
        "const A = () => <><br /><br /></>;",
        "const A = () => <Box><br /><br /></Box>;",
        "const A = () => <p>line<BR /><BR />more</p>;",
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 0,
            "must stay clean for {source}: {:?}",
            result.diagnostics
        );
    }

    for source in [
        "const A = () => <p>line<br />{' '}<br />more</p>;",
        "const A = () => <p>line<br />{/* spacer */}<br />more</p>;",
        "const A = () => <p><><br /><br /></></p>;",
        "const A = () => <p>{cond && <><br /><br /></>}</p>;",
        "const A = () => <p>{items.map((item) => <><br /><br /></>)}</p>;",
        "const A = () => <Box><p><br /><br /></p></Box>;",
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            diagnostic_rules(&result),
            vec!["html/no-consecutive-br"],
            "must keep one lowered warning for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn no_consecutive_br_keeps_parent_state_isolated() {
    let linter = linter_with(Box::new(NoConsecutiveBr));
    let source = "const A = () => <p><br /><br /></p>;\nconst B = () => <p><br /><br /></p>;";
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["html/no-consecutive-br", "html/no-consecutive-br"],
        "each native parent reports independently: {:?}",
        result.diagnostics
    );

    let starts: Vec<u32> = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.start)
        .collect();
    let expected: Vec<u32> = source
        .match_indices("<br")
        .enumerate()
        .filter_map(|(index, (offset, _))| (index % 2 == 1).then_some(offset as u32))
        .collect();
    assert_eq!(starts, expected, "diagnostics must point at second <br>s");
}
