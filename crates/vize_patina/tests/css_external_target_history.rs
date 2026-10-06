//! Complete authored API and report oracles for both original CSS ownership reports.

use serde::Deserialize;
use vize_l0::String;
use vize_patina::rules::css::{CssLintResult, CssLinter, NoDisplayNone};
use vize_patina::{
    HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter, Locale, OutputFormat, format_results,
};

const RULE: &str = "css/no-display-none";
const MESSAGE: &str = "Consider using v-show directive instead of display: none";
const HELP: &str = "v-show toggles visibility without removing from DOM, improving performance for frequent toggles";

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    filename: String,
    source: String,
    style_start: usize,
    style_end: usize,
    expected_starts: Vec<u32>,
    expected_cli: serde_json::Value,
}

fn expected_diagnostic(start: u32) -> LintDiagnostic {
    LintDiagnostic::warn(RULE, MESSAGE, start, start + 7).with_help(HELP)
}

#[test]
fn both_originals_and_subject_controls_preserve_complete_api_and_cli_reports() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("fixtures/issue-7976/cases.json")).unwrap();
    assert_eq!(corpus.cases.len(), 54);
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![RULE.into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full);
    let mut css_linter = CssLinter::new();
    css_linter.add_rule(Box::new(NoDisplayNone));
    for case in corpus.cases {
        let css = case.source.get(case.style_start..case.style_end).unwrap();
        // Empty results must not be an accidental stylesheet-parse fallback.
        let stylesheet = lightningcss::stylesheet::StyleSheet::parse(
            css,
            lightningcss::stylesheet::ParserOptions::default(),
        )
        .unwrap();
        assert!(!stylesheet.rules.0.is_empty(), "{}", case.id);
        let expected = LintResult {
            filename: case.filename.clone(),
            error_count: 0,
            warning_count: case.expected_starts.len(),
            diagnostics: case
                .expected_starts
                .iter()
                .copied()
                .map(expected_diagnostic)
                .collect(),
        };
        let expected_css = CssLintResult {
            error_count: 0,
            warning_count: case.expected_starts.len(),
            diagnostics: case
                .expected_starts
                .iter()
                .map(|start| expected_diagnostic(100 + *start - case.style_start as u32))
                .collect(),
        };
        for _ in 0..3 {
            assert_eq!(
                format!("{:#?}", css_linter.lint(css, 100)),
                format!("{expected_css:#?}"),
                "{}",
                case.id
            );
            let actual = linter.lint_sfc(&case.source, &case.filename);
            assert_eq!(
                format!("{actual:#?}"),
                format!("{expected:#?}"),
                "{}",
                case.id
            );
            let json = format_results(
                &[actual],
                &[(case.filename.clone(), case.source.clone())],
                OutputFormat::Json,
            );
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&json).unwrap(),
                case.expected_cli,
                "{}",
                case.id
            );
        }
        let off = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![RULE.into()]))
            .with_disabled_rules(vec![RULE.into()]);
        let expected_off = LintResult {
            filename: case.filename.clone(),
            diagnostics: vec![],
            error_count: 0,
            warning_count: 0,
        };
        assert_eq!(
            format!("{:#?}", off.lint_sfc(&case.source, &case.filename)),
            format!("{expected_off:#?}")
        );
    }
}
