use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::HeadingLevels;
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
fn heading_levels_runs_over_lowered_markup_ir_once() {
    let source = "const A = () => <><h1>Title</h1><h3>Sub</h3></>;";
    let linter = linter_with(Box::new(HeadingLevels));
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/heading-levels"],
        "migrated heading-levels must report once via lowered markup IR: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);

    let diag = &result.diagnostics[0];
    let h3_start = source.find("<h3").unwrap() as u32;
    assert_eq!(
        diag.start, h3_start,
        "range must start at the skipped JSX heading"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        "<h3>Sub</h3>",
        "range must cover exactly the authored JSX heading"
    );

    let tsx = linter.lint_jsx(
        "const A = (): JSX.Element => <><h1>Title</h1><h3>Sub</h3></>;",
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/heading-levels"],
        "TSX keeps the same lowered markup IR behavior"
    );
}

#[test]
fn heading_levels_reports_each_skip_after_the_first() {
    let source = "const A = () => <><h1>T</h1><h3>S</h3><h6>D</h6></>;";
    let linter = linter_with(Box::new(HeadingLevels));
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);

    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/heading-levels", "a11y/heading-levels"],
        "each heading jump after the first must still report: {:?}",
        result.diagnostics
    );

    let starts: Vec<u32> = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.start)
        .collect();
    assert_eq!(
        starts,
        vec![
            source.find("<h3").unwrap() as u32,
            source.find("<h6").unwrap() as u32,
        ]
    );
}

#[test]
fn heading_levels_preserves_lowered_jsx_root_boundaries() {
    let linter = linter_with(Box::new(HeadingLevels));
    for source in [
        "const A = () => <h1>Title</h1>;\nconst B = () => <h3>Sub</h3>;",
        "const A = () => <><H1>Title</H1><H3>Sub</H3></>;",
        "const A = () => <><h1>Title</h1><svg:h3>Sub</svg:h3></>;",
        "const A = () => <><h1>Title</h1><Headings.h3>Sub</Headings.h3></>;",
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 0,
            "must stay clean for {source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.error_count, 0);
    }
}

#[test]
fn heading_levels_sfc_offsets_template_diagnostics() {
    let source = r#"<script setup lang="ts">
const title = "Title";
</script>

<template>
  <h1>{{ title }}</h1>
  <h3>Sub</h3>
</template>
"#;
    let linter = linter_with(Box::new(HeadingLevels));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        diagnostic_rules(&result),
        vec!["a11y/heading-levels"],
        "SFC template heading skip must report once: {:?}",
        result.diagnostics
    );

    let diag = &result.diagnostics[0];
    let h3_start = source.find("<h3").unwrap() as u32;
    assert_eq!(
        diag.start, h3_start,
        "SFC diagnostic must be offset into file coordinates"
    );
    assert_eq!(
        &source[diag.start as usize..diag.end as usize],
        "<h3>",
        "range must cover the skipped template heading start tag in the full SFC"
    );
}

#[test]
fn heading_levels_sfc_does_not_lint_script_tsx() {
    let source = r#"<script setup lang="tsx">
const A = () => <><h1>Title</h1><h3>Sub</h3></>;
</script>
"#;
    let linter = linter_with(Box::new(HeadingLevels));
    let result = linter.lint_sfc(source, "test.vue");

    assert_eq!(
        result.warning_count, 0,
        "SFC heading-levels must not inspect JSX inside script blocks: {:?}",
        result.diagnostics
    );
    assert_eq!(result.error_count, 0);
}
