use crate::{HelpLevel, LintResult, Linter, Locale, RuleRegistry};
use serde::Deserialize;
use vize_l0::String;

pub const RULE: &str = "a11y/placeholder-label-option";

pub fn configured() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(super::super::super::PlaceholderLabelOption));
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
    let all: Vec<Case> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/lint-history/cases.json"
    )))
    .unwrap();
    assert_eq!(all.len(), 48);
    let selected: Vec<_> = all.into_iter().filter(|case| case.rule == RULE).collect();
    assert_eq!(selected.len(), 4);
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
        "placeholder-label-unrequired-empty-control" => {
            original!("placeholder-label-unrequired-empty-control")
        }
        "placeholder-label-required-empty-control" => {
            original!("placeholder-label-required-empty-control")
        }
        "placeholder-label-required-nonempty-report" => {
            original!("placeholder-label-required-nonempty-report")
        }
        "placeholder-label-optgroup-report" => original!("placeholder-label-optgroup-report"),
        _ => panic!("unregistered original placeholder case"),
    }
}

#[derive(Debug)]
struct Observation {
    initial: LintResult,
    // All four immutable cases have zero fixes; no application is possible.
    applications: Vec<()>,
    unchanged_requery: Option<LintResult>,
}

pub fn assert_original(case: &Case, native: bool) {
    assert_eq!(
        case.history.as_str(),
        "dbab039dde930c513dea7d1358eebde04296661a"
    );
    assert_eq!(case.entry.as_str(), "template");
    assert_eq!(case.rule.as_str(), RULE);
    assert_eq!(case.vue_version, None);
    assert_eq!(case.vapor, None);
    assert_eq!(case.fixes, 0);
    let linter = configured();
    let query = || {
        if native {
            linter
                .lint_native_template(&case.source, &case.filename)
                .unwrap()
        } else {
            linter.lint_template(&case.source, &case.filename)
        }
    };
    let observation = Observation {
        initial: query(),
        applications: vec![],
        unchanged_requery: Some(query()),
    };
    assert_eq!(observation.initial.diagnostics.len(), case.diagnostics);
    assert!(
        observation
            .initial
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.fix.is_none())
    );
    assert!(observation.applications.is_empty());
    assert_eq!(
        format!("{:#?}", observation.initial),
        format!("{:#?}", observation.unchanged_requery.as_ref().unwrap())
    );
    let complete = format!("{case:#?}\n{observation:#?}\n");
    assert_eq!(
        complete,
        snapshot(&case.id).split_once("\n---\n").unwrap().1,
        "{} native={native}",
        case.id
    );
    println!("native-placeholder-history native={native} {complete}");
}

pub fn pair(linter: &Linter, source: &str, findings: usize) {
    let original = linter.lint_template(source, "Pair.vue");
    assert_eq!(original.diagnostics.len(), findings, "{source}");
    assert!(
        original
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.fix.is_none())
    );
    for actual in [
        linter.lint_native_template(source, "Pair.vue").unwrap(),
        linter.lint_template(source, "Pair.vue"),
        linter.lint_native_template(source, "Pair.vue").unwrap(),
    ] {
        assert_eq!(
            format!("{actual:#?}"),
            format!("{original:#?}"),
            "whole output {source}"
        );
    }
}
