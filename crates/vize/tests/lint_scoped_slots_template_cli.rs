#![cfg(test)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

const RULE: &str = "script/no-deprecated-dollar-scopedslots-api";

fn sha256(bytes: &[u8]) -> vize_l0::String {
    vize_l0::cstr!("{:x}", Sha256::digest(bytes))
}

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
        let carrier = vize_l0::cstr!(
            "export default {{ vue: {} }};\n",
            std::str::from_utf8(&config_bytes).unwrap()
        );
        fs::write(directory.path().join("vite.config.mjs"), carrier.as_bytes()).unwrap();
        let original_argv = [
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "--locale",
            "en",
            filename,
        ];
        let mut argv = original_argv;
        argv[2] = "vite.config.mjs";
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(directory.path())
            .args(argv)
            .output()
            .unwrap();
        runs.push(json!({
            "case": case["id"], "profile": profile, "source": source, "config": config,
            "status": output.status.code(), "stdoutBytes": &output.stdout,
            "stderrBytes": &output.stderr,
            "historicalArgv": original_argv, "argv": argv,
            "historicalConfigIdentity": "vize.config.json", "configIdentity": "vite.config.mjs",
            "originalConfigBytes": &config_bytes, "originalConfigSha256": sha256(&config_bytes),
            "configBytes": carrier.as_bytes(), "configSha256": sha256(carrier.as_bytes()),
            "sourceBytes": source.as_bytes(), "sourceSha256": sha256(source.as_bytes()),
            "sourceAfterBytes": fs::read(directory.path().join(filename)).unwrap(),
            "configAfterBytes": fs::read(directory.path().join("vite.config.mjs")).unwrap()
        }));
        fs::write(&evidence, serde_json::to_vec_pretty(&runs).unwrap()).unwrap();
        assert_eq!(
            sha256(source.as_bytes()).as_str(),
            case["sourceSha256"].as_str().unwrap()
        );
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
            fs::read(directory.path().join("vite.config.mjs")).unwrap(),
            carrier.as_bytes()
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
