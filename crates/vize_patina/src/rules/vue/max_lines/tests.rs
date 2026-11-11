use crate::diagnostic::Severity;
use crate::linter::Linter;
use crate::preset::LintPreset;
use vize_l0::config::ConfigLintRuleOptions;

const LONG: &str = "\
<script setup lang=\"ts\">\n\
const label = \"a\";\n\
</script>\n\
\n\
<template>\n\
  <ul>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
    <li>{{ label }}</li>\n\
  </ul>\n\
</template>\n";

fn lint(source: &str, options: serde_json::Value) -> Vec<(u32, u32, String, Severity)> {
    let parsed: ConfigLintRuleOptions = serde_json::from_value(options).unwrap();
    let result = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["max-lines".into()]))
        .with_rule_severity_overrides(vec![("max-lines".into(), Severity::Error)])
        .with_config_rule_options(&parsed)
        .lint_sfc(source, "Long.vue");
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "max-lines")
        .map(|diagnostic| {
            (
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.to_string(),
                diagnostic.severity,
            )
        })
        .collect()
}

#[test]
fn the_issue_file_reports_line_11_when_the_maximum_is_10() {
    let line_11 = LONG
        .split('\n')
        .take(10)
        .map(|line| line.len() + 1)
        .sum::<usize>() as u32;
    assert_eq!(
        lint(LONG, serde_json::json!({ "max-lines": { "max": 10 } })),
        vec![(
            line_11,
            LONG.len() as u32,
            "File has too many lines (18). Maximum allowed is 10.".into(),
            Severity::Error,
        )]
    );
    assert_eq!(
        lint(LONG, serde_json::json!({ "max-lines": 10 })),
        vec![(
            line_11,
            LONG.len() as u32,
            "File has too many lines (18). Maximum allowed is 10.".into(),
            Severity::Error,
        )]
    );
}

#[test]
fn the_default_maximum_stays_silent_on_the_issue_file() {
    let enabled = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec!["max-lines".into()]))
        .lint_sfc(LONG, "Long.vue");
    assert!(
        enabled
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "max-lines"),
        "{:?}",
        enabled.diagnostics
    );
}

#[test]
fn blank_lines_and_comment_only_lines_can_be_skipped() {
    let source = "// note\nconst a = 1;\n\nconst b = 2; // kept\n";
    assert_eq!(
        lint(source, serde_json::json!({ "max-lines": { "max": 2 } })).len(),
        1
    );
    assert!(
        lint(
            source,
            serde_json::json!({ "max-lines": { "max": 2, "skipBlankLines": true, "skipComments": true } })
        )
        .is_empty()
    );
}

#[test]
fn an_unknown_option_is_rejected() {
    let value = serde_json::json!({ "max-lines": { "max": 10, "extra": true } });
    assert!(serde_json::from_value::<ConfigLintRuleOptions>(value).is_err());
}
