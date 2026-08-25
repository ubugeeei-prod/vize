use super::{InspectorLintPlan, InspectorLintPlanItem, inspect_lint_plan};
use std::{collections::BTreeMap, path::Path};
use vize_s0::config::LintRuleSeverity as Severity;

fn item(
    name: &str,
    files: Option<Vec<&str>>,
    ignores: Vec<&str>,
    rules: &[(&str, Severity)],
) -> InspectorLintPlanItem {
    InspectorLintPlanItem {
        name: name.into(),
        base_path: None,
        files: files.map(|patterns| patterns.into_iter().map(Into::into).collect()),
        ignores: ignores.into_iter().map(Into::into).collect(),
        rules: rules
            .iter()
            .map(|(name, severity)| ((*name).into(), *severity))
            .collect::<BTreeMap<_, _>>(),
    }
}

#[test]
fn reports_ordered_winners_and_matches_execution_glob_semantics() {
    let plan = InspectorLintPlan {
        items: vec![
            item(
                "base",
                None,
                vec![],
                &[("base", Severity::Error), ("shared", Severity::Warn)],
            ),
            item(
                "first",
                Some(vec!["src/**/*.{vue,tsx}"]),
                vec![],
                &[("first", Severity::Warn), ("shared", Severity::Error)],
            ),
            item(
                "second",
                Some(vec![
                    "src/**/*.vue",
                    "src/**/*.vue",
                    "!src/**/*.test.vue",
                    "[unterminated",
                    "\0**/*.vue",
                ]),
                vec!["src/generated/**", "!src/generated/keep.vue"],
                &[("second", Severity::Error), ("shared", Severity::Off)],
            ),
        ],
    };
    let payload = inspect_lint_plan(
        &plan,
        Path::new("/project"),
        &[
            "src/App.vue".into(),
            "src/App.test.vue".into(),
            "src/generated/keep.vue".into(),
        ],
    );

    assert_eq!(payload.schema, "vize.inspector.lint-plan");
    assert_eq!(payload.files[0].matched_items, ["base", "first", "second"]);
    let shared = payload.files[0]
        .rules
        .iter()
        .find(|rule| rule.name == "shared")
        .unwrap();
    assert_eq!(shared.severity, Severity::Off);
    assert_eq!(shared.set_by, "second");
    assert_eq!(payload.files[1].matched_items, ["base", "first"]);
    assert_eq!(payload.files[2].matched_items, ["base", "first", "second"]);
}

#[test]
fn distinguishes_global_ignores_from_scoped_ignores_with_rules() {
    let plan = InspectorLintPlan {
        items: vec![
            item("global", None, vec!["generated/**"], &[]),
            item(
                "scoped",
                None,
                vec!["private/**"],
                &[("visible", Severity::Error)],
            ),
        ],
    };
    let payload = inspect_lint_plan(
        &plan,
        Path::new("/project"),
        &[
            "generated/File.vue".into(),
            "private/File.vue".into(),
            "App.vue".into(),
        ],
    );

    assert!(payload.items[0].global_ignore);
    assert!(!payload.items[1].global_ignore);
    assert!(payload.files[0].ignored);
    assert_eq!(payload.files[0].ignored_by, ["global"]);
    assert!(!payload.files[1].ignored);
    assert!(payload.files[1].matched_items.is_empty());
    assert_eq!(payload.files[2].matched_items, ["scoped"]);
}

#[test]
fn invalid_and_empty_file_patterns_fail_closed() {
    let plan = InspectorLintPlan {
        items: vec![
            item("empty", Some(vec![]), vec![], &[("empty", Severity::Error)]),
            item(
                "invalid",
                Some(vec!["[unterminated", "\0**/*.vue"]),
                vec![],
                &[("invalid", Severity::Error)],
            ),
        ],
    };
    let payload = inspect_lint_plan(&plan, Path::new("/project"), &["src/App.vue".into()]);
    assert!(payload.files[0].matched_items.is_empty());
    assert!(payload.files[0].rules.is_empty());
}

#[test]
fn relative_roots_anchor_files_once() {
    let plan = InspectorLintPlan {
        items: vec![item(
            "source",
            Some(vec!["src/**/*.vue"]),
            vec![],
            &[("matched", Severity::Error)],
        )],
    };
    let payload = inspect_lint_plan(&plan, Path::new("project"), &["src/App.vue".into()]);
    assert_eq!(payload.root, "project");
    assert_eq!(payload.files[0].path, "src/App.vue");
    assert_eq!(payload.files[0].matched_items, ["source"]);
}
