#![cfg(test)]

use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const RULE: &str = "script/no-deprecated-dollar-scopedslots-api";

#[test]
fn scoped_slots_template_cli_preserves_whole_packets_and_the_original_n8n_projection() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/lint-scoped-slots-template/cases.json"
    ))
    .unwrap();
    let projection: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/n8n-cli-adoption.json"
    ))
    .unwrap();
    assert_eq!(projection["linter"]["rules"].as_object().unwrap().len(), 51);
    let directory = tempfile::tempdir().unwrap();
    let evidence_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "pr".into()))
        .join("scoped-slots-template-cli");
    fs::create_dir_all(&evidence_root).unwrap();
    let evidence = tempfile::Builder::new()
        .prefix("run-")
        .tempdir_in(evidence_root)
        .unwrap()
        .keep()
        .join("packets.json");
    let mut runs = Vec::new();
    let mut run = |case: &Value, config: Value, expected: Value, profile: &str| {
        let filename = case["filename"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        fs::write(directory.path().join(filename), source).unwrap();
        let config_bytes = serde_json::to_vec(&config).unwrap();
        fs::write(directory.path().join("vize.config.json"), &config_bytes).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(directory.path())
            .args([
                "lint",
                "--config",
                "vize.config.json",
                "--format",
                "json",
                "--locale",
                "en",
                filename,
            ])
            .output()
            .unwrap();
        runs.push(json!({
            "case": case["id"], "profile": profile, "source": source, "config": config,
            "status": output.status.code(), "stdoutBytes": &output.stdout,
            "stderrBytes": &output.stderr
        }));
        fs::write(&evidence, serde_json::to_vec_pretty(&runs).unwrap()).unwrap();
        assert_eq!(
            output.status.code(),
            Some(i32::from(expected[0]["errorCount"].as_u64().unwrap() > 0)),
            "{profile}: {runs:?}"
        );
        assert!(output.stderr.is_empty(), "{profile}: {runs:?}");
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            expected,
            "{profile}: {runs:?}"
        );
        assert_eq!(
            fs::read(directory.path().join(filename)).unwrap(),
            source.as_bytes()
        );
        assert_eq!(
            fs::read(directory.path().join("vize.config.json")).unwrap(),
            config_bytes
        );
        fs::remove_file(directory.path().join(filename)).unwrap();
    };
    for case in corpus["cases"].as_array().unwrap() {
        run(
            case,
            json!({"linter": {"preset": "incremental", "rules": {(RULE): "error"}}}),
            json!([{
                "file": case["filename"], "errorCount": case["cliMessages"].as_array().unwrap().len(),
                "warningCount": 0, "messages": case["cliMessages"]
            }]),
            "selected",
        );
    }
    let case = &corpus["cases"][0];
    run(
        case,
        json!({"linter": projection["linter"]}),
        json!([corpus["origin"]["originalCliExpectation"]]),
        "original-whole51",
    );
    run(
        case,
        json!({"linter": {"preset": "incremental", "rules": {(RULE): "off"}}}),
        json!([{"file": case["filename"], "messages": [], "errorCount": 0, "warningCount": 0}]),
        "explicit-off",
    );
    assert_eq!(runs.len(), 11);
    println!(
        "{}",
        json!({"schema": "vize.lint.scoped-slots-template-cli", "commands": 11, "runs": runs})
    );
}
