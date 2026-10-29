//! Public CLI acceptance for contextual function return types (#11103).
#![cfg(test)]
#![expect(
    clippy::disallowed_macros,
    reason = "fixture assertions use std strings"
)]
#![expect(
    clippy::disallowed_types,
    reason = "fixture assertions use std strings"
)]

use std::{fs, process::Command};

const FIXTURE: &str =
    include_str!("../../vize_patina/tests/fixtures/issue-11103/ContextualCallbacks.vue.fixture");
const FILENAME: &str = "ContextualCallbacks.vue";

fn output_details(output: &std::process::Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn preset_all_keeps_contextually_typed_object_callbacks_clean() {
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join(FILENAME), FIXTURE).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args([
            "lint",
            "--no-config",
            "--preset",
            "all",
            "--format",
            "json",
            FILENAME,
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0), "{}", output_details(&output));
    assert_eq!(output.stderr, b"", "{}", output_details(&output));
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::json!([{
            "file": FILENAME,
            "messages": [],
            "errorCount": 0,
            "warningCount": 0
        }]),
        "{}",
        output_details(&output)
    );
    assert_eq!(
        fs::read(project.path().join(FILENAME)).unwrap(),
        FIXTURE.as_bytes()
    );
}
