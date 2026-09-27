//! Actual public report outputs for selected historical requirements.
//! Legacy output oracles confer no native acceptance or whole-commit credit.

#![expect(
    clippy::disallowed_macros,
    clippy::panic,
    reason = "Insta uses standard formatting and invalid fixture indices must fail closed"
)]

use serde::Deserialize;
use vize_l0::String;
use vize_patina::{
    HelpLevel, LintPreset, LintResult, Linter, Locale, OutputFormat, format_results,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    filename: String,
    source: String,
    rule: String,
    diagnostics: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    history: Vec<String>,
    files: Vec<File>,
    sources: Vec<usize>,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "every field is retained in the complete Debug snapshot"
)]
struct Report {
    results: Vec<LintResult>,
    unchanged_requeries: Vec<LintResult>,
    formatted_messages: Vec<Vec<String>>,
    json: String,
    text: String,
}

fn observe(case: &Case) -> Report {
    let mut results = Vec::new();
    let mut unchanged_requeries = Vec::new();
    let mut formatted_messages = Vec::new();
    for file in &case.files {
        let linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![file.rule.clone()]))
            .with_locale(Locale::En)
            .with_help_level(HelpLevel::Full);
        let result = linter.lint_sfc(&file.source, &file.filename);
        assert_eq!(result.diagnostics.len(), file.diagnostics, "{}", case.id);
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.fix.is_none())
        );
        formatted_messages.push(
            result
                .diagnostics
                .iter()
                .map(|d| d.formatted_message())
                .collect(),
        );
        unchanged_requeries.push(linter.lint_sfc(&file.source, &file.filename));
        results.push(result);
    }
    let sources: Vec<_> = case
        .sources
        .iter()
        .map(|index| {
            let file = case
                .files
                .get(*index)
                .unwrap_or_else(|| panic!("unknown fixture source index: {index}"));
            (file.filename.clone(), file.source.clone())
        })
        .collect();
    let json = format_results(&results, &sources, OutputFormat::Json);
    let text = format_results(&results, &sources, OutputFormat::Text);
    Report {
        results,
        unchanged_requeries,
        formatted_messages,
        json,
        text,
    }
}

#[test]
fn complete_actual_public_reports_and_requeries() {
    let cases: Vec<Case> = serde_json::from_str(include_str!("fixtures/report-history/cases.json"))
        .expect("strict report fixture schema");
    assert_eq!(
        cases.len(),
        4,
        "every reviewed report case remains registered"
    );
    for case in cases {
        for history in &case.history {
            assert_eq!(history.len(), 40, "full historical commit identity");
        }
        let first = observe(&case);
        let repeat = observe(&case);
        let bytes = format!("{case:#?}\n{first:#?}");
        assert_eq!(bytes, format!("{case:#?}\n{repeat:#?}"), "{}", case.id);
        insta::assert_snapshot!(case.id.as_str(), bytes);
    }
}
