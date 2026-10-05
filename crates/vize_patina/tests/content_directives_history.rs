//! Original #8018 and authored complete content-provider regression corpus.

use serde::Deserialize;
use vize_l0::String;
use vize_patina::{
    HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter, Locale, OutputFormat, format_results,
};

const RULE: &str = "html/no-empty-palpable-content";
const HELP: &str = "Add text content, child elements, or use aria-label for accessible content.";
const NOTICE: &str = include_str!("fixtures/issue-8018/Notice.vue.fixture");
const CONTROLS: &str = include_str!("fixtures/issue-8018/Controls.vue.fixture");

#[derive(Deserialize)]
struct Corpus {
    scenarios: Vec<Scenario>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scenario {
    id: String,
    file: String,
    content_directives: Option<Vec<String>>,
    enabled: bool,
    expected_cli: serde_json::Value,
    reported_elements: Vec<String>,
}

#[test]
fn original_and_authored_cases_preserve_entire_findings_and_public_reports() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("fixtures/issue-8018/references.json")).unwrap();
    assert_eq!(corpus.scenarios.len(), 11);
    for case in corpus.scenarios {
        let source = if case.file == "Notice.vue" {
            NOTICE
        } else {
            CONTROLS
        };
        let mut linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![RULE.into()]))
            .with_rule_severity_overrides(vec![(RULE.into(), vize_patina::Severity::Error)])
            .with_locale(Locale::En)
            .with_help_level(HelpLevel::Full);
        if let Some(names) = case.content_directives {
            linter = linter.with_palpable_content_directives(names);
        }
        if !case.enabled {
            linter = linter.with_disabled_rules(vec![RULE.into()]);
        }
        let expected = LintResult {
            filename: case.file.clone(),
            error_count: case.reported_elements.len(),
            warning_count: 0,
            diagnostics: case
                .reported_elements
                .iter()
                .map(|target| {
                    let start = source.find(target.as_str()).unwrap() as u32;
                    // Retained element.loc spans the opening tag, including `>`.
                    let opening = target
                        .split_once("</")
                        .map_or(target.as_str(), |(tag, _)| tag);
                    LintDiagnostic::error(
                        RULE,
                        "<p> element is empty but expects visible content",
                        start,
                        start + opening.len() as u32,
                    )
                    .with_help(HELP)
                })
                .collect(),
        };
        for _ in 0..3 {
            let actual = linter.lint_sfc(source, &case.file);
            assert_eq!(
                format!("{actual:#?}"),
                format!("{expected:#?}"),
                "{}",
                case.id
            );
            let json = format_results(
                &[actual],
                &[(case.file.clone(), source.into())],
                OutputFormat::Json,
            );
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&json).unwrap(),
                case.expected_cli,
                "{}",
                case.id
            );
        }
    }
}

#[test]
fn options_do_not_enable_rules_and_empty_builder_restores_default() {
    let inactive = Linter::with_preset(LintPreset::Incremental)
        .with_palpable_content_directives(vec!["safe-html".into()]);
    assert!(
        inactive
            .lint_sfc(CONTROLS, "Controls.vue")
            .diagnostics
            .is_empty()
    );
    let configured = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_palpable_content_directives(vec!["safe-html".into()]);
    assert!(
        configured
            .lint_sfc(NOTICE, "Notice.vue")
            .diagnostics
            .is_empty()
    );
    let reset = configured.with_palpable_content_directives(vec![]);
    let result = reset.lint_sfc(NOTICE, "Notice.vue");
    assert_eq!(result.diagnostics.len(), 1);
    assert!(
        result
            .diagnostics
            .first()
            .is_some_and(|diagnostic| diagnostic.rule_name == RULE)
    );
}
