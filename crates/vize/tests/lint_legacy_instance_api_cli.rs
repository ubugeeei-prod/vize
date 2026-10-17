//! Whole original, boundary and explicit component-binding CLI contracts.
//! The existing Actions archive owns the binary.
//! Expectations are authored from immutable provider/source and native packets,
//! not captured from a local execution. Print every complete receipt before failing.

#![cfg(test)]
#![expect(
    clippy::disallowed_macros,
    reason = "complete CLI fixtures and receipts use std strings"
)]
#![expect(
    clippy::disallowed_types,
    reason = "complete CLI fixtures and receipts use std strings"
)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, process::Command};

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/"
);
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/cli70.json"
));
const TEMPLATE_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-cli.json"
));
const COMPONENT_BINDINGS_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-component-bindings-cli.json"
));

const COMPONENT_DECLARATION_BINDINGS_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-component-declaration-bindings-cli.json"
));

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn all_original_legacy_api_controls_use_the_actual_cli_and_complete_json_formatter() {
    let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
    assert_eq!(manifest["schema"], "vize.lint.legacy-instance-api-cli70");
    assert_eq!(manifest["version"], 1);
    let mut originals = Vec::new();
    for input in manifest["origin"]["inputs"].as_array().unwrap() {
        let path = std::path::Path::new(FIXTURES).join(input["path"].as_str().unwrap());
        let bytes = fs::read(&path).unwrap();
        assert_eq!(bytes.len() as u64, input["bytes"].as_u64().unwrap());
        assert_eq!(digest(&bytes), input["sha256"].as_str().unwrap());
        originals.push(serde_json::from_slice::<Value>(&bytes).unwrap());
    }
    assert_eq!(originals.len(), 2);
    let independent = originals[0]["records"].as_array().unwrap();
    let native = originals[1]["records"].as_array().unwrap();
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(independent.len(), 70);
    assert_eq!(native.len(), 70);
    assert_eq!(cases.len(), 70);
    assert_eq!(
        cases
            .iter()
            .map(|case| case["id"].as_str().unwrap())
            .collect::<BTreeSet<_>>()
            .len(),
        70,
        "every original case has its own CLI invocation",
    );
    for ((case, provider), native) in cases.iter().zip(independent).zip(native) {
        assert_eq!(case["id"], provider["id"]);
        assert_eq!(case["id"], native["id"]);
        assert_eq!(case["sourceSha256"], provider["sourceSha256"]);
        assert_eq!(case["sourceSha256"], native["sourceSha256"]);
        assert_eq!(
            digest(provider["source"].as_str().unwrap().as_bytes()),
            case["sourceSha256"].as_str().unwrap()
        );
        assert_eq!(
            case["nativeMessageIndices"],
            json!((0..native["messages"].as_array().unwrap().len()).collect::<Vec<_>>()),
            "all native messages are retained in explicit formatter order",
        );
        assert_eq!(
            case["expected"][0]["messages"].as_array().unwrap().len(),
            native["messages"].as_array().unwrap().len()
        );
    }
    run_cli_cases(&manifest, independent, 70);
}

#[test]
fn all_template_authority_boundaries_use_the_actual_cli_and_complete_json_formatter() {
    let mut manifest: Value = serde_json::from_str(TEMPLATE_MANIFEST).unwrap();
    assert_eq!(
        manifest["schema"],
        "vize.lint.legacy-template-authority-cli"
    );
    assert_eq!(manifest["version"], 1);
    let inputs = manifest["origin"]["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 1);
    let input = &inputs[0];
    assert_eq!(input["path"], "template-boundaries.json");
    let bytes = fs::read(std::path::Path::new(FIXTURES).join("template-boundaries.json")).unwrap();
    assert_eq!(bytes.len() as u64, input["bytes"].as_u64().unwrap());
    assert_eq!(digest(&bytes), input["sha256"].as_str().unwrap());
    let boundary: Value = serde_json::from_slice(&bytes).unwrap();
    let sources = boundary["records"].as_array().unwrap();
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(sources.len(), 42);
    assert_eq!(cases.len(), 42);
    assert_eq!(
        cases
            .iter()
            .map(|case| case["id"].as_str().unwrap())
            .collect::<BTreeSet<_>>()
            .len(),
        42,
        "every boundary has its own complete CLI invocation",
    );
    for (case, source) in cases.iter().zip(sources) {
        assert_eq!(case["qualification"], source["qualification"]);
        assert_eq!(
            case["nativeMessageIndices"],
            json!((0..source["nativeMessages"].as_array().unwrap().len()).collect::<Vec<_>>()),
            "retain every native expectation and explicit qualification",
        );
        assert_eq!(
            case["expected"][0]["messages"].as_array().unwrap().len(),
            source["nativeMessages"].as_array().unwrap().len(),
        );
    }
    let overlay: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-authority-review-corrections.json"
    )).unwrap();
    assert_eq!(overlay["frozenInput"]["sha256"], digest(&bytes));
    assert_eq!(
        overlay["frozenCliInput"]["sha256"],
        digest(TEMPLATE_MANIFEST.as_bytes())
    );
    let corrections = overlay["legacyCorrections"].as_array().unwrap();
    assert_eq!(corrections.len(), 6);
    for correction in corrections {
        let source = sources
            .iter()
            .find(|row| row["id"] == correction["id"])
            .unwrap();
        assert_eq!(source["source"], correction["source"]);
        assert_eq!(source["sourceSha256"], correction["sourceSha256"]);
        let case = manifest["cases"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["id"] == correction["id"])
            .unwrap();
        case["historicalExpected"] = case["expected"].clone();
        case["historicalQualification"] = case["qualification"].clone();
        case["expectationAuthority"] = json!("actual-vue3-plain-legacy-attribute-overlay");
        case["qualification"] = json!("product-accuracy-vue3-provider-blindspot");
        case["expected"] = correction["cliExpected"].clone();
        case["status"] = correction["expectedStatus"].clone();
        case["nativeMessageIndices"] =
            json!((0..correction["nativeMessages"].as_array().unwrap().len()).collect::<Vec<_>>());
    }
    run_cli_cases(&manifest, sources, 42);
}

#[test]
fn component_owned_legacy_api_values_preserve_script_warnings_and_complete_cli_packets() {
    let manifest: Value = serde_json::from_str(COMPONENT_BINDINGS_MANIFEST).unwrap();
    assert_eq!(
        manifest["schema"],
        "vize.lint.template-component-bindings-cli"
    );
    assert_eq!(manifest["version"], 1);
    let inputs = manifest["origin"]["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 2);
    assert_eq!(inputs[0]["path"], "template-component-bindings.json");
    assert_eq!(inputs[1]["path"], "template-component-bindings-native.json");
    let mut originals = Vec::new();
    for input in inputs {
        let bytes =
            fs::read(std::path::Path::new(FIXTURES).join(input["path"].as_str().unwrap())).unwrap();
        assert_eq!(bytes.len() as u64, input["bytes"].as_u64().unwrap());
        assert_eq!(digest(&bytes), input["sha256"].as_str().unwrap());
        originals.push(serde_json::from_slice::<Value>(&bytes).unwrap());
    }
    let sources = originals[0]["records"].as_array().unwrap();
    let native = originals[1]["records"].as_array().unwrap();
    assert_eq!(sources.len(), 30);
    assert_eq!(native.len(), 15);
    assert_eq!(manifest["projection"], originals[1]["projection"]);
    let primary = manifest["projection"]["primaryIds"].as_array().unwrap();
    let research = manifest["projection"]["researchOnlyIds"]
        .as_array()
        .unwrap();
    assert_eq!(primary.len(), 15);
    assert_eq!(research.len(), 15);
    let partition: BTreeSet<_> = primary
        .iter()
        .chain(research)
        .map(|id| id.as_str().unwrap())
        .collect();
    assert_eq!(
        partition.len(),
        30,
        "primary and research receipts are disjoint"
    );
    assert_eq!(
        partition,
        sources
            .iter()
            .map(|source| source["id"].as_str().unwrap())
            .collect::<BTreeSet<_>>()
    );
    for id in research {
        let source = sources.iter().find(|source| source["id"] == *id).unwrap();
        assert_eq!(source["surface"], "$scopedSlots");
        assert_eq!(source["productScope"], "unrepaired-scopedSlots-research");
    }
    let source_projection: Vec<_> = primary
        .iter()
        .map(|id| {
            let source = sources.iter().find(|source| source["id"] == *id).unwrap();
            assert_eq!(source["surface"], "$listeners");
            assert_eq!(source["productScope"], "primary-listeners");
            source.clone()
        })
        .collect();
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 15);
    for (((case, source), native), id) in cases
        .iter()
        .zip(&source_projection)
        .zip(native)
        .zip(primary)
    {
        assert_eq!(case["id"], *id);
        assert_eq!(case["id"], native["id"]);
        assert_eq!(case["sourceSha256"], native["sourceSha256"]);
        assert_eq!(case["ownerExpectation"], source["ownerExpectation"]);
        assert_eq!(case["ownerExpectation"], native["ownerExpectation"]);
        assert_eq!(
            case["nativeMessageIndices"],
            json!((0..native["messages"].as_array().unwrap().len()).collect::<Vec<_>>()),
            "all historical script and new template messages remain in the complete packet",
        );
        assert_eq!(
            case["expected"][0]["messages"].as_array().unwrap().len(),
            native["messages"].as_array().unwrap().len()
        );
    }
    run_cli_cases(&manifest, &source_projection, 15);
}

#[test]
fn script_setup_declaration_forms_preserve_complete_cli_packets_and_unqualified_this_frontier() {
    let manifest: Value = serde_json::from_str(COMPONENT_DECLARATION_BINDINGS_MANIFEST).unwrap();
    assert_eq!(
        manifest["schema"],
        "vize.lint.template-component-declaration-bindings-cli"
    );
    assert_eq!(manifest["version"], 1);
    let inputs = manifest["origin"]["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 2);
    assert_eq!(
        inputs[0]["path"],
        "template-component-declaration-bindings.json"
    );
    assert_eq!(
        inputs[1]["path"],
        "template-component-declaration-bindings-native.json"
    );
    let mut originals = Vec::new();
    for input in inputs {
        let bytes =
            fs::read(std::path::Path::new(FIXTURES).join(input["path"].as_str().unwrap())).unwrap();
        assert_eq!(bytes.len() as u64, input["bytes"].as_u64().unwrap());
        assert_eq!(digest(&bytes), input["sha256"].as_str().unwrap());
        originals.push(serde_json::from_slice::<Value>(&bytes).unwrap());
    }
    let sources = originals[0]["records"].as_array().unwrap();
    let native = originals[1]["records"].as_array().unwrap();
    assert_eq!(sources.len(), 40);
    assert_eq!(native.len(), 20);
    assert_eq!(manifest["projection"], originals[1]["projection"]);
    let primary = manifest["projection"]["primaryIds"].as_array().unwrap();
    let research = manifest["projection"]["researchOnlyIds"]
        .as_array()
        .unwrap();
    assert_eq!(primary.len(), 20);
    assert_eq!(research.len(), 20);
    let partition: BTreeSet<_> = primary
        .iter()
        .chain(research)
        .map(|id| id.as_str().unwrap())
        .collect();
    assert_eq!(
        partition.len(),
        40,
        "primary and research receipts are disjoint"
    );
    assert_eq!(
        partition,
        sources
            .iter()
            .map(|source| source["id"].as_str().unwrap())
            .collect::<BTreeSet<_>>()
    );
    for id in research {
        let source = sources.iter().find(|source| source["id"] == *id).unwrap();
        assert_eq!(source["surface"], "$scopedSlots");
        assert_eq!(
            source["productScope"],
            if source["ownerExpectation"] == "bare-setup-owned-template-this-unqualified" {
                "unqualified-template-this-frontier"
            } else {
                "unrepaired-scopedSlots-research"
            }
        );
    }
    let qualified = manifest["projection"]["qualifiedAcceptanceIds"]
        .as_array()
        .unwrap();
    let observation = manifest["projection"]["observationOnlyIds"]
        .as_array()
        .unwrap();
    assert_eq!(qualified.len(), 19);
    assert_eq!(observation, &[json!("listeners-template-this-frontier")]);
    let acceptance_partition: BTreeSet<_> = qualified
        .iter()
        .chain(observation)
        .map(|id| id.as_str().unwrap())
        .collect();
    assert_eq!(acceptance_partition.len(), 20);
    assert_eq!(
        acceptance_partition,
        primary
            .iter()
            .map(|id| id.as_str().unwrap())
            .collect::<BTreeSet<_>>()
    );
    let source_projection: Vec<_> = primary
        .iter()
        .map(|id| {
            let source = sources.iter().find(|source| source["id"] == *id).unwrap();
            assert_eq!(source["surface"], "$listeners");
            assert_eq!(
                source["productScope"],
                if source["ownerExpectation"] == "bare-setup-owned-template-this-unqualified" {
                    "unqualified-template-this-frontier"
                } else {
                    "primary-listeners"
                }
            );
            source.clone()
        })
        .collect();
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 20);
    for (((case, source), native), id) in cases
        .iter()
        .zip(&source_projection)
        .zip(native)
        .zip(primary)
    {
        assert_eq!(case["id"], *id);
        assert_eq!(case["id"], native["id"]);
        assert_eq!(case["sourceSha256"], native["sourceSha256"]);
        assert_eq!(case["ownerExpectation"], source["ownerExpectation"]);
        assert_eq!(case["ownerExpectation"], native["ownerExpectation"]);
        assert_eq!(case["qualification"], native["qualification"]);
        assert_eq!(case["qualifiedAcceptance"], native["qualifiedAcceptance"]);
        if observation.contains(id) {
            assert_eq!(case["qualifiedAcceptance"], false);
            assert_eq!(
                case["qualification"],
                "observation-only-template-this-frontier"
            );
            assert_eq!(case["expected"][0]["messages"], json!([]));
        } else {
            assert!(qualified.contains(id));
            assert_eq!(case["qualifiedAcceptance"], true);
            assert_eq!(case["qualification"], "qualified-primary-listeners");
        }
        assert_eq!(
            case["nativeMessageIndices"],
            json!((0..native["messages"].as_array().unwrap().len()).collect::<Vec<_>>()),
            "all historical script and new template messages remain in the complete packet",
        );
        assert_eq!(
            case["expected"][0]["messages"].as_array().unwrap().len(),
            native["messages"].as_array().unwrap().len()
        );
    }
    run_cli_cases(&manifest, &source_projection, 20);
}

fn run_cli_cases(manifest: &Value, sources: &[Value], expected_count: usize) {
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
    let argv: Vec<_> = manifest["argv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|arg| arg.as_str().unwrap())
        .collect();
    assert_eq!(
        argv,
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

    let project = tempfile::tempdir().unwrap();
    let mut failures = Vec::new();
    let mut captured = 0;
    for (index, (case, provider)) in cases.iter().zip(sources).enumerate() {
        let id = case["id"].as_str().unwrap();
        let source = provider["source"].as_str().unwrap().as_bytes();
        let directory = project.path().join(format!("{index:02}-{id}"));
        let source_path = directory.join("Component.vue");
        let config_path = directory.join("vize.config.json");
        let setup = fs::create_dir(&directory)
            .and_then(|()| fs::write(&source_path, source))
            .and_then(|()| fs::write(&config_path, config));
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
                "expectationAuthority": case["expectationAuthority"],
                "qualifiedAcceptance": case["qualifiedAcceptance"],
                "program": env!("CARGO_BIN_EXE_vize"),
                "actualCwd": directory,
                "argv": argv,
                "sourceSha256": case["sourceSha256"],
                "sourceBytes": source,
                "configSha256": manifest["configSha256"],
                "configBytes": config,
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
        if config_after.as_deref().ok() != Some(config) {
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
