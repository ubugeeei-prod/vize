//! Whole source/API/report contracts for the compatible global ownership slice.

use serde::Deserialize;
use vize_l0::String;
use vize_patina::rules::css::{CssLintResult, CssLinter, NoDisplayNone};
use vize_patina::{
    HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter, Locale, OutputFormat, format_results,
};

const RULE: &str = "css/no-display-none";

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
    expected_css_starts: Vec<u32>,
    expected_cli: serde_json::Value,
    expected_plain: String,
}

fn diagnostic(start: u32) -> LintDiagnostic {
    LintDiagnostic::warn(
        RULE,
        "Consider using v-show directive instead of display: none",
        start,
        start + 7,
    )
    .with_help(
        "v-show toggles visibility without removing from DOM, improving performance for frequent toggles",
    )
}

#[test]
fn global_ownership_preserves_complete_css_sfc_json_plain_and_off_results() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("fixtures/issue-7976-global/cases.json")).unwrap();
    assert_eq!(corpus.cases.len(), 66);
    let configured = |help| {
        Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![RULE.into()]))
            .with_locale(Locale::En)
            .with_help_level(help)
    };
    let linter = configured(HelpLevel::Full);
    let off = configured(HelpLevel::Full).with_disabled_rules(vec![RULE.into()]);
    let mut css_linter = CssLinter::new();
    css_linter.add_rule(Box::new(NoDisplayNone));
    for case in corpus.cases {
        let css = case.source.get(case.style_start..case.style_end).unwrap();
        // No acceptance from a CSS parse refusal returning an empty result.
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
                .map(diagnostic)
                .collect(),
        };
        let expected_css = CssLintResult {
            error_count: 0,
            warning_count: case.expected_css_starts.len(),
            diagnostics: case
                .expected_css_starts
                .iter()
                .map(|start| diagnostic(100 + *start - case.style_start as u32))
                .collect(),
        };
        for _ in 0..3 {
            assert_eq!(
                format!("{:#?}", css_linter.lint(css, 100)),
                format!("{expected_css:#?}"),
                "{} standalone CSS",
                case.id,
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
                case.id,
            );
            let plain = format_results(
                &[linter.lint_sfc(&case.source, &case.filename)],
                &[(case.filename.clone(), case.source.clone())],
                OutputFormat::Plain,
            );
            assert_eq!(plain, case.expected_plain, "{}", case.id);
        }
        let expected_off = LintResult {
            filename: case.filename.clone(),
            diagnostics: vec![],
            error_count: 0,
            warning_count: 0,
        };
        assert_eq!(
            format!("{:#?}", off.lint_sfc(&case.source, &case.filename)),
            format!("{expected_off:#?}"),
            "{} off",
            case.id,
        );
    }
}
