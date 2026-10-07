//! Complete #7927 / #869 projects exercise runtime slot ancestry through the CLI.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "CLI corpus uses std strings")]
#![expect(
    clippy::disallowed_macros,
    reason = "CLI corpus diagnostics use format"
)]

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Catalog {
    issue: u32,
    historical_issue: u32,
    original_command: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    inputs: String,
    outputs: BTreeMap<String, Expected>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    path: String,
    source: String,
    bytes: usize,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    status: i32,
    stdout: String,
    stderr: String,
}
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/slot-provide-inject-7927")
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn inputs(case: &Case) -> Vec<Input> {
    serde_json::from_slice(&fs::read(fixtures().join(&case.inputs)).unwrap()).unwrap()
}
fn preserve_inputs(root: &Path, files: &[Input]) {
    for file in files {
        let source = fs::read(fixtures().join(&file.source)).unwrap();
        assert_eq!(source.len(), file.bytes, "{}", file.path);
        assert_eq!(digest(&source), file.sha256, "{}", file.path);
        assert_eq!(
            fs::read(root.join(&file.path)).unwrap(),
            source,
            "{}",
            file.path
        );
    }
    assert!(!root.join("vize.config.json").exists());
    assert!(!root.join("node_modules").exists());
}
fn catalog() -> Catalog {
    let catalog: Catalog =
        serde_json::from_slice(&fs::read(fixtures().join("cases.json")).unwrap()).unwrap();
    assert_eq!((catalog.issue, catalog.historical_issue), (7927, 869));
    assert_eq!(catalog.cases.len(), 18);
    assert_eq!(
        fs::read_to_string(fixtures().join(&catalog.original_command)).unwrap(),
        "vize lint --cross-file --cross-file-tree --format plain src\n"
    );
    catalog
}

#[test]
fn original_issue_files_and_command_are_preserved_whole() {
    let catalog = catalog();
    let case = catalog
        .cases
        .iter()
        .find(|case| case.name == "original-string")
        .unwrap();
    let original = fs::read_to_string(fixtures().join("issue-7927.md")).unwrap();
    for file in inputs(case) {
        let whole = fs::read_to_string(fixtures().join(&file.source)).unwrap();
        let marker = format!("`{}`:", file.path);
        let (_, section) = original
            .split_once(&marker)
            .expect("original filename section");
        let (_, fenced) = section.split_once("```vue\n").expect("original Vue fence");
        let (fenced, _) = fenced.split_once("```").unwrap();
        assert_eq!(whole, fenced, "complete original {}", file.path);
    }
}

#[test]
fn complete_slot_projects_match_plain_json_and_requested_tree() {
    let catalog = catalog();
    let mut comparisons = 0;
    for case in catalog.cases {
        let files = inputs(&case);
        for format in ["plain", "json"] {
            let expected = &case.outputs[format];
            let expected_stdout = fs::read(fixtures().join(&expected.stdout)).unwrap();
            let expected_stderr = fs::read(fixtures().join(&expected.stderr)).unwrap();
            for _ in 0..2 {
                let root = tempfile::tempdir().unwrap();
                for file in &files {
                    let path = root.path().join(&file.path);
                    fs::create_dir_all(path.parent().unwrap()).unwrap();
                    fs::copy(fixtures().join(&file.source), path).unwrap();
                }
                let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                    .current_dir(root.path())
                    .args([
                        "lint",
                        "--cross-file",
                        "--cross-file-tree",
                        "--format",
                        format,
                        "src",
                    ])
                    .output()
                    .expect("run the current source-built CLI");
                assert_eq!(
                    output.status.code(),
                    Some(expected.status),
                    "{} {format}",
                    case.name
                );
                assert_eq!(
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&expected_stdout),
                    "{} {format}: complete stdout",
                    case.name
                );
                assert_eq!(
                    output.stderr, expected_stderr,
                    "{} {format}: complete stderr",
                    case.name
                );
                if format == "json" {
                    serde_json::from_slice::<serde_json::Value>(&output.stdout)
                        .expect("whole JSON remains valid");
                }
                preserve_inputs(root.path(), &files);
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 72);
    println!(
        "slot provide/inject original CLI corpus: projects=18 formats=2 repetitions=2 comparisons=72 whole_inputs_unchanged=true"
    );
}
