use super::{GlobSequence, LinterRuleResolver};
use crate::config::{
    LintRuleSeverity as Severity, LinterConfig, LinterConfigEntry, LinterConfigPlan,
};
use std::{collections::BTreeMap, fs, path::PathBuf};
use vize_carton::FxHashMap;

fn rules(entries: &[(&str, Severity)]) -> FxHashMap<vize_carton::String, Severity> {
    entries
        .iter()
        .map(|(name, severity)| ((*name).into(), *severity))
        .collect()
}

fn entry(
    base_path: Option<&str>,
    files: Option<Vec<&str>>,
    ignores: Vec<&str>,
    entries: &[(&str, Severity)],
) -> LinterConfigEntry {
    LinterConfigEntry {
        base_path: base_path.map(Into::into),
        files: files.map(|patterns| patterns.into_iter().map(Into::into).collect()),
        ignores: ignores.into_iter().map(Into::into).collect(),
        rules: rules(entries),
    }
}

fn severity_map(config: &LinterConfig) -> BTreeMap<&str, Severity> {
    config
        .rules
        .iter()
        .map(|(name, severity)| (name.as_str(), *severity))
        .collect()
}

#[test]
fn file_globs_preserve_ordered_repeats_and_fail_closed_when_all_are_invalid() {
    let ordered = GlobSequence::new(&[
        "src/**/*.vue".into(),
        "!src/generated/**".into(),
        "src/**/*.vue".into(),
    ]);
    assert!(ordered.matches_files("src/generated/drop.vue"));

    let negative_only = GlobSequence::new(&["!src/generated/**".into()]);
    assert!(negative_only.matches_files("src/App.vue"));
    assert!(!negative_only.matches_files("src/generated/drop.vue"));

    let invalid = GlobSequence::new(&["[unterminated".into(), "\0**/*.vue".into()]);
    assert!(!invalid.matches_files("src/App.vue"));
}

#[test]
fn directory_form_file_globs_select_their_whole_subtree() {
    let directory = GlobSequence::new(&["src/pages".into()]);
    assert!(directory.matches_files("src/pages/index.vue"));
    assert!(directory.matches_files("src/pages/nested/index.vue"));
    assert!(!directory.matches_files("src/components/Card.vue"));
}

#[test]
fn negated_globs_keep_their_marker_when_relative_prefixes_are_stripped() {
    for pattern in ["!./src/generated/**", "!.\\src\\generated\\**"] {
        let sequence = GlobSequence::new(&["src/**/*.vue".into(), pattern.into()]);
        assert!(sequence.matches_files("src/App.vue"), "{pattern}");
        assert!(
            !sequence.matches_files("src/generated/drop.vue"),
            "{pattern}"
        );
    }
}

#[test]
fn resolves_whole_rule_maps_for_each_distinct_glob_set_once() {
    let project = tempfile::tempdir().unwrap();
    for file in [
        "src/components/App.vue",
        "src/components/App.test.vue",
        "src/generated/drop.vue",
        "src/generated/keep.vue",
        "packages/admin/src/Page.vue",
        "packages/admin/src/private/Secret.vue",
        "outside/Other.vue",
    ] {
        let path = project.path().join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }

    let plan = LinterConfigPlan {
        base: LinterConfig {
            rules: rules(&[("base", Severity::Error), ("shared", Severity::Warn)]),
            ..LinterConfig::default()
        },
        entries: vec![
            entry(
                None,
                Some(vec!["src/**/*.{vue,tsx}"]),
                vec![],
                &[("first", Severity::Warn), ("shared", Severity::Error)],
            ),
            entry(
                None,
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
            entry(
                Some("packages\\admin"),
                Some(vec!["src\\**\\*.vue"]),
                vec!["src/private/**"],
                &[("admin", Severity::Warn)],
            ),
            entry(None, Some(vec![]), vec![], &[("empty", Severity::Error)]),
        ],
        global_ignores: vec![],
        rule_options: Default::default(),
    };
    let files = vec![
        PathBuf::from("src/components/App.vue"),
        PathBuf::from("src/components/App.test.vue"),
        PathBuf::from("src/generated/drop.vue"),
        PathBuf::from("src/generated/keep.vue"),
        PathBuf::from("packages\\admin\\src\\Page.vue"),
        PathBuf::from("packages/admin/src/private/Secret.vue"),
        PathBuf::from("outside/Other.vue"),
    ];
    let resolved = LinterRuleResolver::new(plan, project.path(), project.path())
        .resolve_files(&files, project.path());
    let actual = files
        .iter()
        .zip(resolved.file_config_indices.iter())
        .map(|(file, config)| {
            (
                file.to_string_lossy().replace('\\', "/"),
                severity_map(&resolved.configs[*config]),
            )
        })
        .collect::<BTreeMap<_, _>>();

    assert_eq!(
        actual,
        BTreeMap::from([
            (
                "outside/Other.vue".into(),
                BTreeMap::from([("base", Severity::Error), ("shared", Severity::Warn)]),
            ),
            (
                "packages/admin/src/Page.vue".into(),
                BTreeMap::from([
                    ("admin", Severity::Warn),
                    ("base", Severity::Error),
                    ("shared", Severity::Warn),
                ]),
            ),
            (
                "packages/admin/src/private/Secret.vue".into(),
                BTreeMap::from([("base", Severity::Error), ("shared", Severity::Warn)]),
            ),
            (
                "src/components/App.test.vue".into(),
                BTreeMap::from([
                    ("base", Severity::Error),
                    ("first", Severity::Warn),
                    ("shared", Severity::Error),
                ]),
            ),
            (
                "src/components/App.vue".into(),
                BTreeMap::from([
                    ("base", Severity::Error),
                    ("first", Severity::Warn),
                    ("second", Severity::Error),
                    ("shared", Severity::Off),
                ]),
            ),
            (
                "src/generated/drop.vue".into(),
                BTreeMap::from([
                    ("base", Severity::Error),
                    ("first", Severity::Warn),
                    ("shared", Severity::Error),
                ]),
            ),
            (
                "src/generated/keep.vue".into(),
                BTreeMap::from([
                    ("base", Severity::Error),
                    ("first", Severity::Warn),
                    ("second", Severity::Error),
                    ("shared", Severity::Off),
                ]),
            ),
        ]),
    );
    assert_eq!(resolved.configs.len(), 4);
}
