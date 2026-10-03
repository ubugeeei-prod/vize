#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const INPUT: &str = include_str!("fixtures/lint-no-config-schema/App.vue");
const CONFIG: &str = include_str!("fixtures/lint-no-config-schema/vize.config.json");
const EXPECTED: &str = include_str!("fixtures/lint-no-config-schema/expected.json");
const SCHEMA: &str = include_str!("../../../npm/cli/schemas/vize.config.schema.json");

fn write_file(root: &Path, path: &str, contents: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn git(root: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
        ])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn project(with_stale_schema: bool) -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    write_file(root, "src/App.vue", INPUT);
    write_file(root, "vize.config.json", CONFIG);
    if with_stale_schema {
        write_file(root, "node_modules/.vize/vize.config.schema.json", "{}\n");
    }
    git(root, &["init", "--quiet"]);
    git(root, &["add", "--force", "."]);
    git(root, &["commit", "--quiet", "--no-verify", "-m", "fixture"]);
    project
}

fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(root: &Path, folder: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(folder).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path == root.join(".git") {
                continue;
            }
            let kind = entry.file_type().unwrap();
            assert!(!kind.is_symlink(), "unexpected fixture symlink: {path:?}");
            if kind.is_dir() {
                walk(root, &path, files);
            } else {
                assert!(kind.is_file(), "unexpected fixture entry: {path:?}");
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}

fn lint(root: &Path, config_args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["lint", "--format", "json", "--preset", "ecosystem"])
        .args(config_args)
        .arg("src/App.vue")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .output()
        .unwrap()
}

fn assert_diagnostics(output: &Output) {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let expected: serde_json::Value = serde_json::from_str(EXPECTED).unwrap();
    assert_eq!(actual, expected, "{output:?}");
}

#[test]
fn no_config_lint_retains_full_diagnostics_and_fixture_source() {
    for stale in [false, true] {
        let project = project(stale);
        let root = project.path();
        let before_files = inventory(root);
        let before_index = git(root, &["ls-files", "--stage", "-z"]);
        let before_revision = git(root, &["rev-parse", "HEAD"]);
        assert!(git(root, &["status", "--porcelain", "--untracked-files=all"]).is_empty());

        let output = lint(root, &["--no-config"]);

        assert_diagnostics(&output);
        assert_eq!(inventory(root), before_files);
        assert_eq!(git(root, &["ls-files", "--stage", "-z"]), before_index);
        assert_eq!(git(root, &["rev-parse", "HEAD"]), before_revision);
        assert!(git(root, &["status", "--porcelain", "--untracked-files=all"]).is_empty());
    }
}

#[test]
fn configured_lint_keeps_schema_materialization_and_full_diagnostics() {
    let project = project(false);
    let root = project.path();
    let before_index = git(root, &["ls-files", "--stage", "-z"]);
    let before_revision = git(root, &["rev-parse", "HEAD"]);
    let mut expected_files = inventory(root);
    expected_files.insert(
        PathBuf::from("node_modules/.vize/vize.config.schema.json"),
        SCHEMA.as_bytes().to_vec(),
    );

    let output = lint(root, &["--config", "vize.config.json"]);

    assert_diagnostics(&output);
    assert_eq!(inventory(root), expected_files);
    assert_eq!(git(root, &["ls-files", "--stage", "-z"]), before_index);
    assert_eq!(git(root, &["rev-parse", "HEAD"]), before_revision);
    assert_eq!(
        git(root, &["status", "--porcelain", "--untracked-files=all"]),
        b"?? node_modules/.vize/vize.config.schema.json\n"
    );
}
