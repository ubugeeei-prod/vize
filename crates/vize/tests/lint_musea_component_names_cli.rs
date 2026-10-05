#![cfg(test)]

use std::{fs, process::Command};

#[test]
fn original_art_sources_have_complete_empty_default_and_explicit_cli_results() {
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("vize.config.json"), r#"{"linter":{"preset":"incremental","rules":{"vue/component-definition-name-casing":"warn","vue/multi-word-component-names":"error"}}}"#).unwrap();
    for (filename, source) in [
        (
            "my-button.art.vue",
            include_str!(
                "../../vize_patina/tests/fixtures/musea-component-names/my-button.art.vue.txt"
            ),
        ),
        (
            "Badge.art.vue",
            include_str!(
                "../../vize_patina/tests/fixtures/musea-component-names/Badge.art.vue.txt"
            ),
        ),
    ] {
        fs::write(project.path().join(filename), source).unwrap();
        for config in ["--no-config", "--config=vize.config.json"] {
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(project.path())
                .args(["lint", config, "--format", "json", filename])
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(
                actual,
                serde_json::json!([{
                    "file":filename,"messages":[],"errorCount":0,"warningCount":0
                }])
            );
        }
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .args(["lint", "--no-config", "-f", "plain", filename])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(
            output.stdout, b"Patina lint report: No problems found in 1 file(s)\n",
            "{output:?}"
        );
    }
}
