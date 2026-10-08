//! Complete native-driver results for CSS ownership, without a Corsa runtime.

#![cfg(not(target_arch = "wasm32"))]

use serde::Deserialize;
use vize_l0::String;
use vize_patina::{HelpLevel, LintDiagnostic, LintPreset, LintResult, Linter, Locale, Severity};

const CSS: &str = "css/no-display-none";
const PROPS: &str = "type/require-typed-props";

#[derive(Deserialize)]
struct Corpus {
    schema: String,
    version: u32,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    filename: String,
    source: String,
    #[serde(default)]
    extra_rules: Vec<String>,
    expected: Vec<ExpectedDiagnostic>,
}

#[derive(Deserialize)]
struct ExpectedDiagnostic {
    rule: String,
    message: String,
    help: Option<String>,
    start: u32,
    end: u32,
    severity: String,
}

fn configured(case: &Case) -> Linter {
    let mut rules = vec![CSS.into(), PROPS.into()];
    rules.extend(case.extra_rules.iter().cloned());
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(rules))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full)
        .with_corsa_path(Some(
            std::env::temp_dir().join("vize-no-corsa-css-ownership-runtime"),
        ))
}

fn expected(case: &Case, css_severity: Option<Severity>) -> LintResult {
    let diagnostics: Vec<_> = case
        .expected
        .iter()
        .filter(|diagnostic| diagnostic.rule != CSS || css_severity.is_some())
        .map(|diagnostic| {
            let rule = match diagnostic.rule.as_str() {
                CSS => CSS,
                PROPS => PROPS,
                "script/no-get-current-instance" => "script/no-get-current-instance",
                "parser/template" => "parser/template",
                rule => panic!("unexpected authored rule {rule}"),
            };
            let severity = if rule == CSS {
                css_severity.unwrap()
            } else if diagnostic.severity == "error" {
                Severity::Error
            } else {
                assert_eq!(diagnostic.severity, "warning");
                Severity::Warning
            };
            let mut result = if severity == Severity::Error {
                LintDiagnostic::error(
                    rule,
                    diagnostic.message.clone(),
                    diagnostic.start,
                    diagnostic.end,
                )
            } else {
                LintDiagnostic::warn(
                    rule,
                    diagnostic.message.clone(),
                    diagnostic.start,
                    diagnostic.end,
                )
            };
            if let Some(help) = &diagnostic.help {
                result = result.with_help(help.clone());
            }
            result
        })
        .collect();
    LintResult {
        filename: case.filename.clone(),
        error_count: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count(),
        warning_count: diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count(),
        diagnostics,
    }
}

#[test]
fn native_type_aware_css_ownership_preserves_complete_results_and_severity() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("fixtures/issue-7976-type-aware/cases.json")).unwrap();
    assert_eq!(corpus.schema, "vize.css.type-aware-ownership.corpus");
    assert_eq!(corpus.version, 1);
    assert_eq!(corpus.cases.len(), 16);
    for case in corpus.cases {
        for severity in [Severity::Warning, Severity::Error] {
            let linter =
                configured(&case).with_rule_severity_overrides(vec![(CSS.into(), severity)]);
            let expected = expected(&case, Some(severity));
            for _ in 0..3 {
                let actual = linter.lint_sfc(&case.source, &case.filename);
                assert_eq!(
                    format!("{actual:#?}"),
                    format!("{expected:#?}"),
                    "{} {severity:?}",
                    case.id,
                );
            }
        }
        // Disabling CSS leaves the native type rule active: this is still the
        // driver route, including scriptless and empty-query early returns.
        let off = configured(&case).with_disabled_rules(vec![CSS.into()]);
        assert_eq!(
            format!("{:#?}", off.lint_sfc(&case.source, &case.filename)),
            format!("{:#?}", expected(&case, None)),
            "{} disabled CSS",
            case.id,
        );
    }
}
