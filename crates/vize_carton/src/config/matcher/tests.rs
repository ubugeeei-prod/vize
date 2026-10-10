use super::{LintPlanScope, ProjectIgnoreSet};
use std::path::Path;

fn scope(files: Option<&[&str]>, ignores: &[&str]) -> LintPlanScope {
    let files: Option<Vec<crate::String>> =
        files.map(|patterns| patterns.iter().copied().map(Into::into).collect());
    let ignores = ignores
        .iter()
        .copied()
        .map(Into::into)
        .collect::<Vec<crate::String>>();
    let root = Path::new("/");
    LintPlanScope::new(None, files.as_deref(), &ignores, root, root)
}

#[test]
fn escaped_metacharacters_match_literals_in_files_and_ignores() {
    let root = Path::new("/");
    for (pattern, matched, unmatched) in [
        (
            "pages/users/\\[id\\].vue",
            "/pages/users/[id].vue",
            "/pages/users/id.vue",
        ),
        (
            "pages/users/\\].vue",
            "/pages/users/].vue",
            "/pages/users/x.vue",
        ),
        (
            "pages/users/\\*.vue",
            "/pages/users/*.vue",
            "/pages/users/id.vue",
        ),
        (
            "pages/users/\\?.vue",
            "/pages/users/?.vue",
            "/pages/users/a.vue",
        ),
        (
            "pages/users/\\{id\\}.vue",
            "/pages/users/{id}.vue",
            "/pages/users/id.vue",
        ),
        (
            "pages/users/[[]id].vue",
            "/pages/users/[id].vue",
            "/pages/users/id.vue",
        ),
    ] {
        let files = scope(Some(&[pattern]), &[]);
        assert!(files.matches(root.join(matched).as_path()), "{pattern}");
        assert!(!files.matches(root.join(unmatched).as_path()), "{pattern}");
        let ignored = scope(None, &[pattern]);
        assert!(ignored.ignores(root.join(matched).as_path()), "{pattern}");
        assert!(
            !ignored.ignores(root.join(unmatched).as_path()),
            "{pattern}"
        );
    }

    let windows = scope(Some(&[r"src\**\*.vue"]), &[]);
    assert!(windows.matches(Path::new("/src/components/App.vue")));
    assert!(!windows.matches(Path::new("/packages/App.vue")));

    let negated = scope(Some(&[r"src/**/*.vue", r"!.\src\generated\**"]), &[]);
    assert!(negated.matches(Path::new("/src/App.vue")));
    assert!(!negated.matches(Path::new("/src/generated/drop.vue")));
}

#[test]
fn project_ignore_sequences_preserve_root_negation_and_literal_metacharacters() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path().join("app");
    let absolute = root.join("absolute/**");
    let outside = project.path().join("outside/**");
    let ignores = [
        "generated/**",
        "!generated/KeepItem.vue",
        r"src/\[id\].vue",
        "node_modules/**",
        "!node_modules/keep.vue",
        absolute.to_str().unwrap(),
        "!absolute/KeepItem.vue",
        outside.to_str().unwrap(),
    ]
    .into_iter()
    .map(|pattern| crate::config::ConfigEntryIgnore {
        base_path: Some(root.to_string_lossy().as_ref().into()),
        pattern: pattern.into(),
    })
    .collect::<Vec<_>>();
    let set = ProjectIgnoreSet::new(&ignores, project.path()).unwrap();
    let actual = [
        "app/generated/DropItem.vue",
        "app/generated/KeepItem.vue",
        "app/src/[id].vue",
        "app/src/id.vue",
        "generated/DropItem.vue",
        "app/src/node_modules/drop.vue",
        "app/node_modules/keep.vue",
        "app/absolute/DropItem.vue",
        "app/absolute/KeepItem.vue",
        "outside/DropItem.vue",
    ]
    .into_iter()
    .map(|name| (name, set.is_ignored(&project.path().join(name))))
    .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            ("app/generated/DropItem.vue", true),
            ("app/generated/KeepItem.vue", false),
            ("app/src/[id].vue", true),
            ("app/src/id.vue", false),
            ("generated/DropItem.vue", false),
            ("app/src/node_modules/drop.vue", true),
            ("app/node_modules/keep.vue", false),
            ("app/absolute/DropItem.vue", true),
            ("app/absolute/KeepItem.vue", false),
            ("outside/DropItem.vue", true),
        ]
    );
}
