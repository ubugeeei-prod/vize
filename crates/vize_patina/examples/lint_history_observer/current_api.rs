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
pub(super) struct Case {
    pub(super) id: String,
    pub(super) history: String,
    pub(super) source: String,
    pub(super) filename: String,
    pub(super) entry: String,
    pub(super) rule: String,
    pub(super) vue_version: Option<String>,
    pub(super) vapor: Option<bool>,
    pub(super) diagnostics: usize,
    pub(super) fixes: usize,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "every observation field is retained in the complete Debug oracle"
)]
pub(super) struct Application {
    pub(super) diagnostic_index: usize,
    pub(super) source: vize_l0::String,
    pub(super) result: LintResult,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "every observation field is retained in the complete Debug oracle"
)]
pub(super) struct Observation {
    pub(super) initial: LintResult,
    pub(super) applications: Vec<Application>,
    pub(super) unchanged_requery: Option<LintResult>,
}

fn query(linter: &Linter, case: &Case, source: &str) -> LintResult {
    match case.entry.as_str() {
        "template" => linter.lint_template(source, &case.filename),
        "script" => linter.lint_script(source, &case.filename),
        "sfc" => linter.lint_sfc(source, &case.filename),
        entry => panic!("unknown fixture entry: {entry}"),
    }
}

pub(super) fn configured(case: &Case) -> Linter {
    let version = case.vue_version.as_deref().map(|version| match version {
        "2" => VueVersion::V2,
        "3" => VueVersion::V3,
        version => panic!("unknown fixture Vue version: {version}"),
    });
    Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![case.rule.as_str().into()]))
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full)
        .with_vue_version(version)
        .with_vapor_mode(case.vapor)
}

fn observe(case: &Case) -> Observation {
    let linter = configured(case);
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

pub(super) fn capture(input: &str) -> Result<String, serde_json::Error> {
    let case: Case = serde_json::from_str(input)?;
    assert_eq!(case.history.len(), 40, "full historical commit identity");
    let observation = observe(&case);
    Ok(format!("{case:#?}\n{observation:#?}\n").into())
}
