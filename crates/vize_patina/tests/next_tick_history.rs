//! Complete current public API oracles for selected legacy fix requirements.
//! These snapshots are legacy regression evidence; they confer no native credit.

#![expect(
    clippy::panic,
    clippy::disallowed_macros,
    reason = "tests panic on invalid fixtures and snapshot the complete actual Debug bytes"
)]

use serde::Deserialize;
use vize_l0::{String, config::VueVersion};
use vize_patina::{HelpLevel, LintPreset, LintResult, Linter, Locale};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    history: String,
    source: String,
    filename: String,
    entry: String,
    rule: String,
    vue_version: Option<String>,
    vapor: Option<bool>,
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
    source: vize_l0::String,
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

fn query(linter: &Linter, case: &Case, source: &str) -> LintResult {
    match case.entry.as_str() {
        "template" => linter.lint_template(source, &case.filename),
        "script" => linter.lint_script(source, &case.filename),
        "sfc" => linter.lint_sfc(source, &case.filename),
        entry => panic!("unknown fixture entry: {entry}"),
    }
}

fn observe(case: &Case) -> Observation {
    let version = case.vue_version.as_deref().map(|version| match version {
        "2" => VueVersion::V2,
        "3" => VueVersion::V3,
        version => panic!("unknown fixture Vue version: {version}"),
    });
    let linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![case.rule.as_str().into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full)
        .with_vue_version(version)
        .with_vapor_mode(case.vapor);
    let initial = query(&linter, case, &case.source);
    assert_eq!(initial.diagnostics.len(), case.diagnostics, "{}", case.id);
    let applications: Vec<_> = initial
        .diagnostics
        .iter()
        .enumerate()
        .filter_map(|(diagnostic_index, diagnostic)| {
            diagnostic.fix.as_ref().map(|fix| {
                // Every offered fix runs independently on the original bytes.
                let source = fix.apply(&case.source);
                let result = query(&linter, case, &source);
                Application {
                    diagnostic_index,
                    source,
                    result,
                }
            })
        })
        .collect();
    assert_eq!(applications.len(), case.fixes, "{}", case.id);
    let unchanged_requery = applications
        .is_empty()
        .then(|| query(&linter, case, &case.source));
    Observation {
        initial,
        applications,
        unchanged_requery,
    }
}

#[test]
fn full_current_results_and_independent_fix_applications() {
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("fixtures/next-tick-history/cases.json"))
            .expect("strict fixture schema");
    assert_eq!(
        cases.len(),
        10,
        "every reviewed case must remain registered"
    );
    for case in cases {
        assert_eq!(case.history.len(), 40, "full historical commit identity");
        // Snapshot all public fields via Debug; no filtering or normalization.
        let first = observe(&case);
        let repeat = observe(&case);
        let first_bytes = format!("{case:#?}\n{first:#?}");
        assert_eq!(
            first_bytes,
            format!("{case:#?}\n{repeat:#?}"),
            "{}",
            case.id
        );
        insta::assert_snapshot!(case.id.as_str(), first_bytes);
    }
}
