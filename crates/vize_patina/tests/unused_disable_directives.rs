//! `--report-unused-disable-directives` for template comments.

use vize_patina::{LintPreset, Linter};

fn lint(source: &str, report: bool) -> Vec<(&'static str, String, String)> {
    Linter::with_preset(LintPreset::Opinionated)
        .with_report_unused_disable_directives(report)
        .lint_sfc(source, "App.vue")
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            let span = source
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default()
                .to_owned();
            (diagnostic.rule_name, diagnostic.message.to_string(), span)
        })
        .collect()
}

fn template(body: &str) -> String {
    [
        "<script setup>\nconst html = '';\n</script>\n<template>\n",
        body,
        "\n</template>\n",
    ]
    .concat()
}

#[test]
fn a_used_pragma_is_not_reported() {
    let source =
        template("  <!-- eslint-disable-next-line vue/no-v-html -->\n  <div v-html=\"html\" />");
    assert_eq!(lint(&source, true), []);
}

#[test]
fn an_unused_listed_rule_is_reported_at_the_pragma() {
    let source = template("  <!-- eslint-disable-next-line vue/no-v-html -->\n  <div />");
    assert_eq!(
        lint(&source, true),
        [(
            "vue/comment-directive",
            "Unused eslint-disable directive (no problems were reported from 'vue/no-v-html')."
                .to_owned(),
            "eslint-disable-next-line vue/no-v-html".to_owned(),
        )],
    );
    assert_eq!(lint(&source, false), [], "reporting is opt-in");
}

#[test]
fn a_pragma_without_rules_is_reported_when_it_suppressed_nothing() {
    let source = template("  <!-- eslint-disable-next-line -->\n  <div />");
    let found = lint(&source, true);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(
        found[0].1,
        "Unused eslint-disable directive (no problems were reported)."
    );
}

#[test]
fn rules_this_run_does_not_execute_are_left_to_their_linter() {
    let source = template(
        "  <!-- eslint-disable-next-line @typescript-eslint/no-explicit-any, vue/no-v-html -->\n  <div v-html=\"html\" />",
    );
    assert_eq!(lint(&source, true), []);
    let disabled = Linter::with_preset(LintPreset::Opinionated)
        .with_disabled_rules(vec!["vue/no-v-html".into()])
        .with_report_unused_disable_directives(true)
        .lint_sfc(
            &template("  <!-- eslint-disable-next-line vue/no-v-html -->\n  <div />"),
            "App.vue",
        );
    assert_eq!(disabled.diagnostics.len(), 0);
}

#[test]
fn a_region_reports_only_the_rules_that_never_fired() {
    let source = template(
        "  <!-- eslint-disable vue/no-v-html, vue/require-v-for-key -->\n  <div v-html=\"html\" />\n  <!-- eslint-enable -->",
    );
    let found = lint(&source, true);
    assert_eq!(found.len(), 1, "{found:?}");
    let message = &found[0].1;
    assert!(
        message.split_once("'vue/require-v-for-key'").is_some(),
        "{found:?}"
    );
}
