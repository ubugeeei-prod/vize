//! Independent reported public-instance, generic-attrs and event-key contracts.
#![expect(
    clippy::unwrap_used,
    reason = "fixture setup and assertions after raw capture"
)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use serde_json::{Value, json};
use vize_l0::{String, cstr};

use super::super::{TSGO_ENV, install_mapper_manifest, install_vue_package, workspace_root};

#[path = "../reported_dynamic_targets/capture.rs"]
mod capture;

const INPUTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/reported-component-contracts-11445/inputs.json"
));
const CAPTURE: &str = "VIZE_TEST_REPORTED_COMPONENT_CONTRACTS_CAPTURE_DIR";
const REQUIRED: &str = "VIZE_TEST_REPORTED_COMPONENT_CONTRACTS_REQUIRED";

type Files = BTreeMap<String, String>;

struct Case {
    issue: u64,
    role: &'static str,
    files: Files,
    expected: Expected,
}

enum Expected {
    Clean,
    Exact(String),
    UnknownProp { line: usize, column: usize },
    MissingProp { line: usize, column: usize },
}

fn changed(source: &str, from: &str, to: &str) -> String {
    assert_eq!(source.matches(from).count(), 1, "derived control preimage");
    source.replacen(from, to, 1).into()
}

fn edit(files: &Files, file: &str, from: &str, to: &str) -> Files {
    let mut result = files.clone();
    result.insert(file.into(), changed(&files[file], from, to));
    result
}

fn anchor(source: &str, token: &str) -> (usize, usize) {
    assert_eq!(
        source.matches(token).count(),
        1,
        "unique authored error anchor"
    );
    let offset = source.find(token).unwrap();
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() + 1;
    (line, column)
}

fn error(files: &Files, file: &str, token: &str, code: u32, message: &str) -> Expected {
    let (line, column) = anchor(&files[file], token);
    Expected::Exact(cstr!(
        "{file}({line},{column}): error TS{code}: {message}\n"
    ))
}

fn push(cases: &mut Vec<Case>, issue: u64, role: &'static str, files: Files, expected: Expected) {
    cases.push(Case {
        issue,
        role,
        files,
        expected,
    });
}

fn original(inputs: &Value, issue: u64) -> Files {
    inputs["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["issue"] == issue)
        .unwrap()["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            let text = file["text"].as_str().unwrap();
            assert_eq!(file["bytes"], text.len());
            assert_eq!(
                file["sha256"].as_str(),
                Some(capture::hash(text.as_bytes()).as_str())
            );
            (
                String::from(file["path"].as_str().unwrap()),
                String::from(text),
            )
        })
        .collect()
}

fn cases(inputs: &Value) -> Vec<Case> {
    let mut cases = Vec::new();
    let files = original(inputs, 11445);
    let bad = edit(
        &files,
        "src/Probe.ts",
        "export const Bar",
        "export const Baz: number = \"baz\";\nexport const Bar",
    );
    for (role, current) in [
        ("original", files.clone()),
        ("authored-error", bad.clone()),
        ("repair", files.clone()),
        ("authored-error-restored", bad),
    ] {
        let expected = if role == "authored-error" || role == "authored-error-restored" {
            error(
                &current,
                "src/Probe.ts",
                "Baz",
                2322,
                "Type 'string' is not assignable to type 'number'.",
            )
        } else {
            Expected::Clean
        };
        push(&mut cases, 11445, role, current, expected);
    }
    let props = edit(
        &files,
        "src/Foo.vue",
        "<template>",
        "<script setup lang=\"ts\">\ndefineProps<{ value: number }>();\n</script>\n<template>",
    );
    let no_any = edit(
        &props,
        "src/Probe.ts",
        "export const Bar",
        "declare const instance: InstanceType<typeof Foo>;\nexport const invalid: string = instance.$props.value;\nexport const Bar",
    );
    let expected = error(
        &no_any,
        "src/Probe.ts",
        "invalid",
        2322,
        "Type 'number' is not assignable to type 'string'.",
    );
    push(&mut cases, 11445, "declared-prop-not-any", no_any, expected);
    let plain = edit(
        &files,
        "src/augment.d.ts",
        "[key: `$${string}`]: unknown;",
        "[key: string]: unknown;",
    );
    push(
        &mut cases,
        11445,
        "plain-index-control",
        plain,
        Expected::Clean,
    );

    let files = original(inputs, 11446);
    let wrong = edit(&files, "src/Probe.vue", "bar=\"a\" p0", ":bar=\"1\"");
    for (role, current) in [
        ("original", files.clone()),
        ("wrong-prop", wrong.clone()),
        ("repair", files.clone()),
        ("wrong-prop-restored", wrong),
    ] {
        let expected = if role == "wrong-prop" || role == "wrong-prop-restored" {
            error(
                &current,
                "src/Probe.vue",
                "bar=",
                2322,
                "Type 'number' is not assignable to type 'string'.",
            )
        } else {
            Expected::Clean
        };
        push(&mut cases, 11446, role, current, expected);
    }
    for (role, options) in [
        ("explicit-open", json!({"checkUnknownProps":false})),
        ("explicit-closed", json!({"checkUnknownProps":true})),
        ("strict-closed", json!({"strictTemplates":true})),
    ] {
        let mut current = files.clone();
        let mut config: Value = serde_json::from_str(&current["tsconfig.json"]).unwrap();
        config["vueCompilerOptions"] = options;
        current.insert(
            "tsconfig.json".into(),
            serde_json::to_string_pretty(&config).unwrap().into(),
        );
        let expected = if role == "explicit-open" {
            Expected::Clean
        } else {
            let (line, column) = anchor(&current["src/Probe.vue"], "p0");
            Expected::UnknownProp { line, column }
        };
        push(&mut cases, 11446, role, current, expected);
    }
    let missing = edit(&files, "src/Probe.vue", "bar=\"a\" p0", "p0");
    let (line, column) = anchor(&missing["src/Probe.vue"], "Foo p0");
    push(
        &mut cases,
        11446,
        "required-prop-control",
        missing,
        Expected::MissingProp { line, column },
    );

    let files = original(inputs, 11453);
    let wrong = edit(&files, "src/Probe.vue", "value.toFixed()", "value.nope()");
    for (role, current) in [
        ("original", files.clone()),
        ("wrong-payload", wrong.clone()),
        ("repair", files.clone()),
        ("wrong-payload-restored", wrong),
    ] {
        let expected = if role == "wrong-payload" || role == "wrong-payload-restored" {
            error(
                &current,
                "src/Probe.vue",
                "nope",
                2339,
                "Property 'nope' does not exist on type 'number'.",
            )
        } else {
            Expected::Clean
        };
        push(&mut cases, 11453, role, current, expected);
    }
    let camel = edit(&files, "src/Probe.vue", "@bar-baz=", "@barBaz=");
    push(
        &mut cases,
        11453,
        "camel-spelling",
        camel.clone(),
        Expected::Clean,
    );
    let camel_bad = edit(&camel, "src/Probe.vue", "value.toFixed()", "value.nope()");
    let expected = error(
        &camel_bad,
        "src/Probe.vue",
        "nope",
        2339,
        "Property 'nope' does not exist on type 'number'.",
    );
    push(
        &mut cases,
        11453,
        "camel-wrong-payload",
        camel_bad,
        expected,
    );
    let distinct = edit(&files, "src/Foo.vue", "\"bar-baz\":", "bazBar:");
    push(
        &mut cases,
        11453,
        "noncolliding-events",
        distinct,
        Expected::Clean,
    );
    cases
}

fn assert_case(case: &Case, packet: &Value) {
    let process = &packet["checker"];
    assert!(process["error"].is_null(), "{packet}");
    assert!(process["signal"].is_null(), "{packet}");
    assert_eq!(process["stderrBytes"], json!([]), "{packet}");
    let stdout = process["stdoutUtf8"]
        .as_str()
        .expect("complete UTF-8 native output");
    match &case.expected {
        Expected::Clean => {
            assert_eq!(process["status"], 0, "{packet}");
            assert_eq!(stdout, "", "{packet}");
        }
        Expected::Exact(expected) => {
            assert_eq!(process["status"], 2, "{packet}");
            assert_eq!(stdout, expected.as_str(), "{packet}");
        }
        Expected::MissingProp { line, column } => {
            assert_eq!(process["status"], 2, "{packet}");
            let prefix = cstr!("src/Probe.vue({line},{column}): error TS2345: Argument of type '");
            assert!(stdout.starts_with(prefix.as_str()), "{packet}");
            assert!(
                stdout.contains("Property 'bar' is missing in type "),
                "{packet}"
            );
            assert_eq!(stdout.matches(": error TS").count(), 1, "{packet}");
        }
        Expected::UnknownProp { line, column } => {
            assert_eq!(process["status"], 2, "{packet}");
            let prefix = cstr!(
                "src/Probe.vue({line},{column}): error TS2353: Object literal may only specify known properties, and '\"p0\"' does not exist in type '"
            );
            assert!(
                stdout.starts_with(prefix.as_str()) && stdout.ends_with("'.\n"),
                "{packet}"
            );
            assert_eq!(stdout.lines().count(), 1, "{packet}");
        }
    }
    let companion = &packet["mapperCompanion"];
    assert_eq!(companion["process"]["status"], 0, "{packet}");
    assert!(companion["process"]["error"].is_null(), "{packet}");
    assert_eq!(companion["process"]["stderrBytes"], json!([]), "{packet}");
    assert!(companion["protocolError"].is_null(), "{packet}");
    assert!(
        companion["responses"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r.get("error").is_none()),
        "{packet}"
    );
}

#[test]
fn originals_and_authored_error_repair_restore_keep_component_contracts() {
    let required = std::env::var_os(REQUIRED);
    assert!(required.is_none() || required.as_deref() == Some(std::ffi::OsStr::new("1")));
    let Some(host) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        assert!(required.is_none(), "{TSGO_ENV} is required");
        eprintln!("skipping unconfigured component-contract native host");
        return;
    };
    assert!(host.is_file(), "{TSGO_ENV} must name the actual host");
    let capture_override = std::env::var_os(CAPTURE).map(PathBuf::from);
    assert!(
        required.is_none() || capture_override.is_some(),
        "persistent capture override required in the mandatory role"
    );
    // The full workflow calls this child in both the early syntax group and
    // the later project group. Keep both raw populations without overwriting.
    let capture_dir = Some(capture_override.unwrap_or_else(|| {
        workspace_root()
            .join("target/vize-tests/content-mapper-directive-owners/reported-component-contracts")
            .join(std::process::id().to_string())
    }));
    if let Some(path) = &capture_dir {
        std::fs::create_dir_all(path).unwrap();
    }
    let inputs: Value = serde_json::from_str(INPUTS).unwrap();
    let cases = cases(&inputs);
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let mut packets = Vec::new();
    let mut providers = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let project = tempfile::Builder::new()
            .prefix("reported-component-contracts-")
            .tempdir_in(&root)
            .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        for (name, text) in &case.files {
            let path = project.path().join(name.as_str());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text.as_bytes()).unwrap();
        }
        if index == 0 {
            providers.push(capture::provider(&host, project.path()));
        }
        let args = [
            "--runExternalCode",
            "--noEmit",
            "-p",
            "tsconfig.json",
            "--pretty",
            "false",
        ];
        let packet = json!({"issue":case.issue,"role":case.role,"cwd":project.path(),"argv":args,"inputs":case.files,
            "checker":capture::process(Command::new(&host).current_dir(project.path()).args(args),None),
            "mapperCompanion":capture::mapper_companion(project.path(),&case.files),"publicAcceptance":false});
        if let Some(path) = &capture_dir {
            capture::save(
                &path.join(cstr!("{index:02}-{}-{}.json", case.issue, case.role).as_str()),
                &packet,
            );
        }
        packets.push(packet);
        if index + 1 == cases.len() {
            providers.push(capture::provider(&host, project.path()));
        }
    }
    let envelope = json!({"schema":"vize.reportedComponentContracts.native.v1","inputs":inputs,"providers":providers,"cases":packets,"semanticVerdictNotYetApplied":true,"publicAcceptance":false});
    if let Some(path) = &capture_dir {
        capture::save(&path.join("whole-capture.json"), &envelope);
    }
    println!("REPORTED_COMPONENT_CONTRACTS_RAW={envelope}");
    let providers = envelope["providers"].as_array().unwrap();
    for key in ["checkerImage", "sourceCliImage"] {
        assert!(providers[0][key]["sha256"].is_string(), "{envelope}");
        assert_eq!(
            providers[0][key]["sha256"], providers[1][key]["sha256"],
            "{envelope}"
        );
    }
    assert_eq!(
        providers[0]["vueManifestBytes"], providers[1]["vueManifestBytes"],
        "{envelope}"
    );
    for provider in providers {
        assert_eq!(
            provider["checkerVersionCompanion"]["status"], 0,
            "{envelope}"
        );
        assert_eq!(
            provider["checkerVersionCompanion"]["stderrBytes"],
            json!([]),
            "{envelope}"
        );
    }
    for (case, packet) in cases.iter().zip(envelope["cases"].as_array().unwrap()) {
        assert_case(case, packet);
    }
}
