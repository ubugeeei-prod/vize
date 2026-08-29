use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::MouseEventsHaveKeyEvents;
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
fn mouse_events_preserves_standard_jsx_clean_boundary() {
    let linter = linter_with(Box::new(MouseEventsHaveKeyEvents));
    for source in [
        "const A = () => <div onMouseEnter={show} />;",
        "const A = () => <div onMouseLeave={hide} />;",
        "const A = () => <div onMouseEnterCapture={show} />;",
        "const A = () => <div onMouseLeaveCapture={hide} />;",
        "const A = () => <Tooltip onMouseenterCapture={show} />;",
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
fn mouse_events_runs_over_lowered_markup_ir_once() {
    let source = "const A = () => <div onMouseenterCapture={show} />;";
    let linter = linter_with(Box::new(MouseEventsHaveKeyEvents));
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/mouse-events-have-key-events"],
        "migrated mouse event rule must report once via lowered markup IR: {:?}",
        result.diagnostics
    );

    let diag = &result.diagnostics[0];
    let div_start = source.find("<div").unwrap() as u32;
    assert_eq!(diag.start, div_start, "range must start at the JSX tag");

    let tsx = linter.lint_jsx(
        "const A = (): JSX.Element => <div onMouseenterCapture={show} />;",
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/mouse-events-have-key-events"],
        "TSX keeps the same lowered markup IR behavior"
    );
}

#[test]
fn mouse_events_jsx_lowering_keeps_legacy_companion_mapping() {
    let linter = linter_with(Box::new(MouseEventsHaveKeyEvents));
    let directive = linter.lint_jsx(
        "const A = () => <div v-on:mouseenter={show} />;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        diagnostic_rules(&directive),
        vec!["a11y/mouse-events-have-key-events"],
        "JSX v-on directive spelling keeps the old fallback warning"
    );

    let jsx_focus = linter.lint_jsx(
        "const A = () => <div v-on:mouseenter={show} onFocus={show} />;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        diagnostic_rules(&jsx_focus),
        vec!["a11y/mouse-events-have-key-events"],
        "standard JSX onFocus does not satisfy legacy v-on:focus"
    );

    let directive_pair = linter.lint_jsx(
        "const A = () => <div v-on:mouseenter={show} v-on:focus={show} />;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        directive_pair.warning_count, 0,
        "v-on:focus companion produced by legacy lowering must stay clean: {:?}",
        directive_pair.diagnostics
    );

    let paired = linter.lint_jsx(
        "const A = () => <div onMouseenterCapture={show} onFocusCapture={show} />;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        paired.warning_count, 0,
        "focus companion produced by legacy lowering must stay clean: {:?}",
        paired.diagnostics
    );

    let both_missing = linter.lint_jsx(
        "const A = () => <div onMouseenterCapture={show} onMouseleaveCapture={hide} />;",
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        diagnostic_rules(&both_missing),
        vec![
            "a11y/mouse-events-have-key-events",
            "a11y/mouse-events-have-key-events"
        ],
        "missing focus and blur still produce both diagnostics"
    );
}
