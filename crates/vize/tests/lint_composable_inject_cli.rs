//! Complete #10916 projects prove composable injects through the public CLI.
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "whole CLI fixtures")]
#![expect(clippy::disallowed_types, reason = "whole CLI fixtures")]

use serde::Deserialize;
use serde_json::{Value, json};
use std::{fs, path::Path, path::PathBuf, process::Command};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u32,
    issue: u32,
    command: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    root: String,
    status: i32,
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/composable-inject-10916")
}

fn manifest() -> Manifest {
    serde_json::from_slice(&fs::read(fixture_root().join("manifest.json")).unwrap()).unwrap()
}

fn materialize(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let input = entry.path();
        if input.is_dir() {
            materialize(&input, &destination.join(entry.file_name()));
            continue;
        }
        let authored = entry.file_name();
        let authored = authored.to_string_lossy();
        let runtime = authored.strip_suffix(".txt").expect("source fixture suffix");
        fs::copy(input, destination.join(runtime)).unwrap();
    }
}

fn run(case: &Case) -> (std::process::Output, Value) {
    let project = tempfile::tempdir().unwrap();
    materialize(&fixture_root().join(&case.root), project.path());
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--format",
            "json",
            "--no-config",
            "--cross-file",
            "src/**/*.vue",
        ])
        .output()
        .expect("run source-built vize lint");
    let report = serde_json::from_slice(&output.stdout).expect("JSON lint report");
    (output, report)
}

fn exact_packet(report: &Value) -> Value {
    let mut files = report
        .as_array()
        .expect("file reports")
        .iter()
        .map(|file| {
            let messages = file["messages"]
                .as_array()
                .expect("message array")
                .iter()
                .map(|message| {
                    json!({
                        "ruleId": message["ruleId"],
                        "ruleDocsPath": message["ruleDocsPath"],
                        "severity": message["severity"],
                        "message": message["message"],
                        "line": message["line"],
                        "column": message["column"],
                        "endLine": message["endLine"],
                        "endColumn": message["endColumn"]
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "file": file["file"],
                "messages": messages,
                "errorCount": file["errorCount"],
                "warningCount": file["warningCount"]
            })
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left["file"].as_str().cmp(&right["file"].as_str()));
    Value::Array(files)
}

#[test]
fn called_composable_inject_matches_and_missing_provider_reports_at_the_component() {
    let manifest = manifest();
    assert_eq!((manifest.schema, manifest.issue), (1, 10916));
    assert_eq!(
        manifest.command,
        "vize lint -f json --no-config --cross-file \"src/**/*.vue\""
    );
    assert_eq!(manifest.cases.len(), 2);

    for case in &manifest.cases {
        let (output, report) = run(case);
        assert_eq!(output.status.code(), Some(case.status), "{}", case.name);
        let expected = match case.name.as_str() {
            "matched-slot-provider" => json!([
                {"file":"src/App.vue","messages":[],"errorCount":0,"warningCount":0},
                {"file":"src/components/MyForm.vue","messages":[],"errorCount":0,"warningCount":0},
                {"file":"src/components/MyInput.vue","messages":[],"errorCount":0,"warningCount":0}
            ]),
            "missing-provider" => json!([{
                "file": "src/components/MyInput.vue",
                "messages": [{
                    "ruleId": "cross-file",
                    "ruleDocsPath": "docs/content/rules/cross-file.md",
                    "severity": 2,
                    "message": "[vize:cross-file] vize:croquis/cf/unmatched-inject: **Unmatched Inject**: `inject(RequiredKey)` has no matching `provide()` in any ancestor component and returns `undefined` at runtime.",
                    "line": 4,
                    "column": 1,
                    "endLine": 4,
                    "endColumn": 14
                }],
                "errorCount": 1,
                "warningCount": 0
            }]),
            other => panic!("unknown case: {other}"),
        };
        assert_eq!(exact_packet(&report), expected, "{}", case.name);
    }
}
