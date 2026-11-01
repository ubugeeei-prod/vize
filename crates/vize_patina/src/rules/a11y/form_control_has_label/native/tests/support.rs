use crate::{HelpLevel, LintResult, Linter, Locale, RuleRegistry};
use serde::Deserialize;
use vize_l0::String;

pub const RULE: &str = "a11y/form-control-has-label";

pub fn configured() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(super::super::super::FormControlHasLabel));
    Linter::with_registry(registry)
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::Full)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub history: String,
    pub source: String,
    pub filename: String,
    pub entry: String,
    pub rule: String,
    pub vue_version: Option<String>,
    pub vapor: Option<bool>,
    pub diagnostics: usize,
    pub fixes: usize,
}

pub fn cases() -> Vec<Case> {
    let cases: Vec<Case> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/lint-history/cases.json"
    )))
    .unwrap();
    assert_eq!(cases.len(), 48);
    let selected: Vec<_> = cases.into_iter().filter(|case| case.rule == RULE).collect();
    assert_eq!(selected.len(), 6);
    selected
}

fn snapshot(id: &str) -> &'static str {
    macro_rules! original {
        ($id:literal) => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/snapshots/integration__lint_history__",
                $id,
                ".snap"
            ))
        };
    }
    match id {
        "form-label-hidden-input-report-9751" => original!("form-label-hidden-input-report-9751"),
        "form-label-hidden-select-control" => original!("form-label-hidden-select-control"),
        "form-label-hidden-textarea-control" => original!("form-label-hidden-textarea-control"),
        "form-label-until-found-control" => original!("form-label-until-found-control"),
        "form-label-bound-hidden-control" => original!("form-label-bound-hidden-control"),
        "form-label-visible-input-control" => original!("form-label-visible-input-control"),
        _ => panic!("unregistered frozen case"),
    }
}

#[derive(Debug)]
#[expect(dead_code, reason = "complete original Debug oracle fields")]
struct Application {
    diagnostic_index: usize,
    source: String,
    result: LintResult,
}

#[derive(Debug)]
#[expect(dead_code, reason = "complete original Debug oracle fields")]
struct Observation {
    initial: LintResult,
    applications: Vec<Application>,
    unchanged_requery: Option<LintResult>,
}

pub fn assert_original(case: &Case, native: bool) {
    assert_eq!(
        case.history.as_str(),
        "708c7fd03f9cfd399d81a246d016abda7e095929"
    );
    assert_eq!(case.entry.as_str(), "template");
    assert_eq!(case.vue_version, None);
    assert_eq!(case.vapor, None);
    assert_eq!(case.fixes, 0);
    // Physically consume the newly pinned copy of this exact original input;
    // the immutable Case+Observation still supplies every expected byte.
    let source = if case.id == "form-label-bound-hidden-control" {
        let pinned = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/native-input-label-history-6881/BoundHiddenInput.vue.txt"
        ));
        assert_eq!(pinned, case.source.as_str());
        pinned
    } else {
        case.source.as_str()
    };
    let linter = configured();
    let query = || {
        if native {
            linter.lint_native_template(source, &case.filename).unwrap()
        } else {
            linter.lint_template(source, &case.filename)
        }
    };
    let initial = query();
    assert_eq!(initial.diagnostics.len(), case.diagnostics);
    assert!(
        initial
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.fix.is_none())
    );
    let observation = Observation {
        initial,
        applications: vec![],
        unchanged_requery: Some(query()),
    };
    let complete = format!("{case:#?}\n{observation:#?}\n");
    let original = snapshot(&case.id).split_once("\n---\n").unwrap().1;
    assert_eq!(complete, original, "complete {} native={native}", case.id);
    println!("native-form-label-history native={native} {complete}");
}

pub fn pair(linter: &Linter, source: &str, findings: usize) {
    let legacy = linter.lint_template(source, "NewPair.vue");
    assert_eq!(legacy.diagnostics.len(), findings, "legacy {source}");
    assert!(
        legacy
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.fix.is_none())
    );
    let original = format!("{legacy:#?}");
    for result in [
        linter.lint_native_template(source, "NewPair.vue").unwrap(),
        linter.lint_template(source, "NewPair.vue"),
        linter.lint_native_template(source, "NewPair.vue").unwrap(),
    ] {
        assert_eq!(format!("{result:#?}"), original, "whole output {source}");
    }
}
