//! Execute every legacy instance API case and retain its complete current and historical receipt.

use super::vite_config;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

pub(crate) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn run_cli_cases(manifest: &Value, sources: &[Value], expected_count: usize) {
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), expected_count);
    assert_eq!(sources.len(), expected_count);
    for (case, source) in cases.iter().zip(sources) {
        assert_eq!(case["id"], source["id"]);
        assert_eq!(case["sourceSha256"], source["sourceSha256"]);
        assert_eq!(
            digest(source["source"].as_str().unwrap().as_bytes()),
            case["sourceSha256"].as_str().unwrap()
        );
    }
    let config = manifest["config"].as_str().unwrap().as_bytes();
    assert_eq!(
        config.len() as u64,
        manifest["configBytes"].as_u64().unwrap()
    );
    assert_eq!(digest(config), manifest["configSha256"].as_str().unwrap());
    assert_eq!(
        serde_json::from_slice::<Value>(config).unwrap(),
        json!({"linter": {"preset": "incremental", "rules": {
            "script/no-deprecated-dollar-listeners-api": "error",
            "script/no-deprecated-events-api": "error"
        }}}),
        "empty incremental registry plus exactly the two explicitly selected rules",
    );
    let historical_argv: Vec<_> = manifest["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect();
    assert_eq!(
        historical_argv,
        [
            "lint",
            "Component.vue",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "--help-level",
            "full"
        ]
    );
    let mut argv = historical_argv.clone();
    argv[3] = "vite.config.mjs";
    let carrier = vite_config::source(config);
    let carrier_bytes = carrier.as_bytes();

    let project = tempfile::tempdir().unwrap();
    let mut failures = Vec::new();
    let mut captured = 0;
    for (index, (case, provider)) in cases.iter().zip(sources).enumerate() {
        let id = case["id"].as_str().unwrap();
        let source = provider["source"].as_str().unwrap().as_bytes();
        let directory = project.path().join(format!("{index:02}-{id}"));
        let source_path = directory.join("Component.vue");
        let config_path = directory.join("vite.config.mjs");
        let setup = fs::create_dir(&directory)
            .and_then(|()| fs::write(&source_path, source))
            .and_then(|()| fs::write(&config_path, carrier_bytes));
        let output = setup.and_then(|()| {
            Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(&directory)
                .args(&argv)
                .output()
        });
        let source_after = fs::read(&source_path);
        let config_after = fs::read(&config_path);
        println!(
            "{}",
            json!({
                "schema": "vize.lint.legacy-instance-api-cli.receipt",
                "suite": manifest["schema"],
                "case": id,
                "qualification": case["qualification"],
                "historicalQualification": case["historicalQualification"],
                "historicalExpected": case["historicalExpected"],
                "historicalNativeMessageIndices": case["historicalNativeMessageIndices"],
                "expectationAuthority": case["expectationAuthority"],
                "qualifiedAcceptance": case["qualifiedAcceptance"],
                "program": env!("CARGO_BIN_EXE_vize"),
                "actualCwd": directory,
                "argv": argv,
                "historicalArgv": historical_argv,
                "sourceSha256": case["sourceSha256"],
                "sourceBytes": source,
                "configSha256": digest(carrier_bytes),
                "configBytes": carrier_bytes,
                "originalConfigSha256": manifest["configSha256"],
                "originalConfigBytes": config,
                "configIdentity": {"historical": "vize.config.json", "current": "vite.config.mjs"},
                "status": output.as_ref().ok().and_then(|result| result.status.code()),
                "statusText": output.as_ref().ok().map(|result| format!("{}", result.status)),
                "stdoutBytes": output.as_ref().ok().map(|result| &result.stdout),
                "stderrBytes": output.as_ref().ok().map(|result| &result.stderr),
                "executionError": output.as_ref().err().map(|error| format!("{error}")),
                "sourceAfterBytes": source_after.as_ref().ok(),
                "sourceAfterError": source_after.as_ref().err().map(|error| format!("{error}")),
                "configAfterBytes": config_after.as_ref().ok(),
                "configAfterError": config_after.as_ref().err().map(|error| format!("{error}")),
                "expectedStatus": case["status"],
                "expectedStderrBytes": case["stderrBytes"],
                "expected": case["expected"],
            })
        );
        captured += 1;
        if source_after.as_deref().ok() != Some(source) {
            failures.push(format!("{id}: source bytes changed or could not be read"));
        }
        if config_after.as_deref().ok() != Some(carrier_bytes) {
            failures.push(format!(
                "{id}: selected config bytes changed or could not be read"
            ));
        }
        let output = match output {
            Ok(output) => output,
            Err(error) => {
                failures.push(format!("{id}: could not execute actual CLI: {error}"));
                continue;
            }
        };
        if output.status.code().map(i64::from) != case["status"].as_i64() {
            failures.push(format!("{id}: complete CLI status differs"));
        }
        if serde_json::to_value(&output.stderr).unwrap() != case["stderrBytes"] {
            failures.push(format!("{id}: complete CLI stderr differs"));
        }
        match serde_json::from_slice::<Value>(&output.stdout) {
            Ok(actual) if actual == case["expected"] => {}
            Ok(_) => failures.push(format!("{id}: complete JSON formatter packet differs")),
            Err(error) => failures.push(format!("{id}: complete stdout is not JSON: {error}")),
        }
    }
    println!(
        "{}",
        json!({"schema": "vize.lint.legacy-instance-api-cli.summary", "suite": manifest["schema"], "captured": captured, "failures": failures})
    );
    assert_eq!(
        captured, expected_count,
        "every original control has a complete receipt"
    );
    assert!(
        failures.is_empty(),
        "all {expected_count} CLI receipts were captured before this aggregate failure:\n{}",
        failures.join("\n")
    );
}

#[test]
fn callback_return_receivers_stay_silent_in_all_six_whole_cli_packets() {
    let manifest: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/callback-return-ownership-cli.json"
    )))
    .unwrap();
    assert_eq!(
        manifest["schema"],
        "vize.lint.callback-return-ownership-cli"
    );
    assert_eq!(manifest["version"], 1);
    for input in manifest["frozenInputs"].as_array().unwrap() {
        let bytes =
            fs::read(std::path::Path::new(super::FIXTURES).join(input["path"].as_str().unwrap()))
                .unwrap();
        assert_eq!(bytes.len() as u64, input["bytes"].as_u64().unwrap());
        assert_eq!(digest(&bytes), input["sha256"].as_str().unwrap());
    }
    let sources = manifest["sources"].as_array().unwrap();
    assert_eq!(
        sources[0]["sourceSha256"],
        manifest["ownership"]["originalSourceSha256"]
    );
    assert_eq!(
        manifest["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|case| { case["expected"][0]["messages"].as_array().unwrap().len() })
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 0, 0, 0],
        "a call result is not a proven Vue instance; $emit stays valid"
    );
    run_cli_cases(&manifest, sources, 6);
}
