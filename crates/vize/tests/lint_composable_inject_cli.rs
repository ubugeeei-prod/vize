//! Complete #10916 projects prove composable injects through the public CLI.
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "whole CLI fixtures")]
#![expect(clippy::disallowed_types, reason = "whole CLI fixtures")]

use serde::Deserialize;
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};

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

fn run(case: &Case) -> (std::process::Output, Value) {
    let root = fixture_root().join(&case.root);
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
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

fn messages(report: &Value) -> Vec<&Value> {
    report
        .as_array()
        .expect("file reports")
        .iter()
        .flat_map(|file| file["messages"].as_array().into_iter().flatten())
        .collect()
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
        assert!(
            report
                .as_array()
                .unwrap()
                .iter()
                .all(|file| !file["file"].as_str().unwrap().ends_with(".ts")),
            "autoloaded composables are analysis inputs, not extra lint targets"
        );
        let cross_file = messages(&report)
            .into_iter()
            .filter(|message| message["ruleId"] == "cross-file")
            .collect::<Vec<_>>();
        match case.name.as_str() {
            "matched-slot-provider" => assert!(
                cross_file.iter().all(|message| {
                    let text = message["message"].as_str().unwrap_or_default();
                    !text.contains("unused-provide") && !text.contains("unmatched-inject")
                }),
                "matched provider must have no provide/inject finding: {cross_file:?}"
            ),
            "missing-provider" => {
                let unmatched = cross_file
                    .iter()
                    .filter(|message| {
                        message["message"]
                            .as_str()
                            .is_some_and(|text| text.contains("unmatched-inject"))
                    })
                    .collect::<Vec<_>>();
                assert_eq!(unmatched.len(), 1, "{cross_file:?}");
                assert_eq!(unmatched[0]["line"], 4);
            }
            other => panic!("unknown case: {other}"),
        }
    }
}
