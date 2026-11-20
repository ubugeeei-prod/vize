//! Complete current public API oracles for legacy linter fixes that need host
//! configuration: presets, rule sets, severity and category overrides, and
//! typed rule options. Each case lints one pinned source with the configured
//! linter and compares the complete Debug observation with its pinned oracle.
//! These oracles are legacy regression evidence; they confer no native credit.

#![expect(
    clippy::panic,
    clippy::disallowed_macros,
    reason = "tests panic on invalid fixtures and compare the complete actual Debug bytes"
)]

use std::path::{Path, PathBuf};

use serde::Deserialize;
use vize_l0::{String, config::ConfigLintRuleOptions};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Locale, Severity};

const ROOT: &str = "../../tests/_fixtures/differential/linter/config-history-6881";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    history: String,
    source_file: String,
    filename: String,
    entry: String,
    preset: String,
    enabled_rules: Option<Vec<String>>,
    disabled_rules: Vec<String>,
    rule_severities: Vec<(String, String)>,
    disabled_categories: Vec<String>,
    category_severities: Vec<(String, String)>,
    rule_options: serde_json::Value,
    diagnostics: usize,
    fixes: usize,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "every observation field is retained in the complete Debug oracle"
)]
struct Application {
    diagnostic_index: usize,
    source: String,
    result: LintResult,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "every observation field is retained in the complete Debug oracle"
)]
struct Observation {
    initial: LintResult,
    applications: Vec<Application>,
    unchanged_requery: Option<LintResult>,
}

fn severity(value: &str) -> Severity {
    match value {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        value => panic!("unknown fixture severity: {value}"),
    }
}

fn configured(case: &Case) -> Linter {
    let preset = LintPreset::parse(&case.preset)
        .unwrap_or_else(|| panic!("unknown fixture preset: {}", case.preset));
    let options: ConfigLintRuleOptions = serde_json::from_value(case.rule_options.clone())
        .unwrap_or_else(|error| panic!("{}: invalid rule options: {error}", case.id));
    Linter::with_preset(preset)
        .with_enabled_rules(case.enabled_rules.clone())
        .with_disabled_rules(case.disabled_rules.clone())
        .with_rule_severity_overrides(
            case.rule_severities
                .iter()
                .map(|(rule, level)| (rule.clone(), severity(level)))
                .collect(),
        )
        .with_disabled_categories(case.disabled_categories.clone())
        .with_category_severity_overrides(
            case.category_severities
                .iter()
                .map(|(category, level)| (category.clone(), severity(level)))
                .collect(),
        )
        .with_config_rule_options(&options)
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full)
}

fn query(linter: &Linter, case: &Case, source: &str) -> LintResult {
    match case.entry.as_str() {
        "template" => linter.lint_template(source, &case.filename),
        "script" => linter.lint_script(source, &case.filename),
        "sfc" => linter.lint_sfc(source, &case.filename),
        entry => panic!("unknown fixture entry: {entry}"),
    }
}

fn observe(case: &Case, source: &str) -> Observation {
    let linter = configured(case);
    let initial = query(&linter, case, source);
    assert_eq!(initial.diagnostics.len(), case.diagnostics, "{}", case.id);
    let applications: Vec<_> = initial
        .diagnostics
        .iter()
        .enumerate()
        .filter_map(|(diagnostic_index, diagnostic)| {
            diagnostic.fix.as_ref().map(|fix| {
                // Every offered fix runs independently on the original bytes.
                let fixed = fix.apply(source);
                let result = query(&linter, case, &fixed);
                Application {
                    diagnostic_index,
                    source: fixed,
                    result,
                }
            })
        })
        .collect();
    assert_eq!(applications.len(), case.fixes, "{}", case.id);
    let unchanged_requery = applications
        .is_empty()
        .then(|| query(&linter, case, source));
    Observation {
        initial,
        applications,
        unchanged_requery,
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(ROOT)
}

fn read(path: &Path) -> std::string::String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn configured_current_results_and_independent_fix_applications() {
    let cases: Vec<Case> =
        serde_json::from_str(&read(&root().join("cases.json"))).expect("strict fixture schema");
    assert_eq!(
        cases.len(),
        56,
        "every reviewed configured case must remain registered"
    );
    // Regenerate the oracles with VIZE_CONFIG_HISTORY_UPDATE=1.
    let update = std::env::var_os("VIZE_CONFIG_HISTORY_UPDATE").is_some();
    for case in &cases {
        assert_eq!(case.history.len(), 40, "full historical commit identity");
        let source = read(&root().join(case.source_file.as_str()));
        let first = observe(case, &source);
        let repeat = observe(case, &source);
        let first_bytes = format!("{case:#?}\n{first:#?}\n");
        assert_eq!(
            first_bytes,
            format!("{case:#?}\n{repeat:#?}\n"),
            "{}",
            case.id
        );
        let oracle = root().join(format!("{}.observation.txt", case.id));
        if update {
            std::fs::write(&oracle, &first_bytes).expect("write oracle");
        }
        assert_eq!(read(&oracle), first_bytes, "{}", case.id);
    }
}
