//! Whole source CLI packets for the authored selected-alias collector corpus.
#![cfg(test)]
#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use vize_l0::{String, cstr};

const ROOT_MARKER: &str = "__VIZE_ALIAS_ROOT__";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Matrix {
    input_files: Vec<String>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    project: String,
    repair_of: Option<String>,
    exit_code: i32,
}

struct Run {
    id: String,
    expected_exit: i32,
    expected_stdout: String,
    output: Result<Output, String>,
    before: Value,
    after: Value,
}

fn digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

#[test]
fn capture_digest_keeps_complete_standard_sha256_vectors() {
    assert_eq!(
        digest(b"").as_str(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        digest(b"abc").as_str(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

fn file_digest(path: &Path) -> String {
    digest(&std::fs::read(path).unwrap())
}

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn source_sha(workspace: &Path) -> String {
    let result = Command::new("git")
        .current_dir(workspace)
        .args(["--no-replace-objects", "rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "source checkout identity is required"
    );
    let sha = std::str::from_utf8(&result.stdout).unwrap().trim();
    assert!(sha.len() == 40 && sha.bytes().all(|byte| byte.is_ascii_hexdigit()));
    sha.into()
}

fn input_manifest(root: &Path, names: &[String]) -> Value {
    json!(
        names
            .iter()
            .map(|name| {
                let bytes = std::fs::read(root.join(name.as_str())).unwrap();
                json!({"file":name,"bytes":bytes.len(),"sha256":digest(&bytes)})
            })
            .collect::<Vec<_>>()
    )
}

/// Bind only complete, quoted compiler-path values; no actual packet is masked.
fn expected_stdout(template: &str, root: &Path) -> String {
    let report: Value = serde_json::from_str(template).unwrap();
    let paths = report["programs"][0]["compilerOptions"]["paths"]
        .as_object()
        .unwrap();
    let mut bound = String::from(template);
    let mut count = 0;
    for targets in paths.values() {
        for target in targets.as_array().unwrap() {
            let target = target.as_str().unwrap();
            let relative = target
                .strip_prefix(cstr!("{ROOT_MARKER}/").as_str())
                .unwrap();
            assert!(!relative.is_empty() && !Path::new(relative).is_absolute());
            assert!(!relative.split('/').any(|part| part == ".."));
            let quoted = serde_json::to_string(target).unwrap();
            count += template.matches(&quoted).count();
            let actual = root.join(relative);
            let actual = actual.to_str().unwrap().replace('\\', "/");
            bound = bound
                .replace(&quoted, &serde_json::to_string(&actual).unwrap())
                .into();
        }
    }
    assert_eq!(template.matches(ROOT_MARKER).count(), count);
    assert_eq!(bound.matches(ROOT_MARKER).count(), 0);
    bound
}

#[cfg(unix)]
fn signal(output: &Output) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    output.status.signal()
}

#[cfg(not(unix))]
fn signal(_output: &Output) -> Option<i32> {
    None
}

#[test]
fn selected_aliases_keep_all_authored_cli_packets_and_cold_repairs() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .canonicalize()
        .unwrap();
    let native = corsa_requirement::required_or_skip(corsa_path::resolve(&workspace));
    if std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some() {
        assert!(
            native.is_some(),
            "required CLI corpus must execute the native engine"
        );
    }
    let Some(native) = native else {
        return;
    };
    let native = Path::new(&native).canonicalize().unwrap();
    let cli = Path::new(env!("CARGO_BIN_EXE_vize"))
        .canonicalize()
        .unwrap();
    let fixture =
        workspace.join("tests/_fixtures/differential/typecheck/selected-alias-collector-3984");
    let matrix: Matrix =
        serde_json::from_slice(&std::fs::read(fixture.join("cases.json")).unwrap()).unwrap();
    let frozen_inputs: Value =
        serde_json::from_slice(&std::fs::read(fixture.join("input-manifest.json")).unwrap())
            .unwrap();
    assert_eq!(matrix.cases.len(), 8);
    assert_eq!(matrix.input_files.len(), 7);

    let parent = std::env::var_os("VIZE_CANON_ALIAS_CAPTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| tempfile::tempdir().unwrap().keep());
    std::fs::create_dir_all(&parent).unwrap();
    let capture = parent
        .canonicalize()
        .unwrap()
        .join(cstr!("selected-alias-collector-{}", std::process::id()).as_str());
    std::fs::create_dir(&capture).unwrap();
    std::fs::create_dir(capture.join("projects")).unwrap();
    std::fs::create_dir(capture.join("cases")).unwrap();
    let sha = source_sha(&workspace);
    let collector = workspace.join("crates/vize/src/commands/check/imports_aliases.rs");
    let authority = json!({
        "sourceSha":sha,"sourceCli":cli,"sourceCliSha256":file_digest(&cli),
        "nativePath":native,"nativeSha256":file_digest(&native),
        "collectorSha256":file_digest(&collector),
        "testSha256":file_digest(&workspace.join("crates/vize/tests/check_canon_selected_alias_cli.rs")),
        "casesSha256":file_digest(&fixture.join("cases.json")),
        "inputManifestSha256":file_digest(&fixture.join("input-manifest.json"))
    });
    write_json(
        &capture.join("receipt.json"),
        &json!({
            "success":false,"complete":false,"authority":authority,"scope":"eight cold source CLI packets"
        }),
    );

    let mut runs = Vec::new();
    let mut records = Vec::new();
    // No diagnostic, stdout, stderr or exit oracle is asserted until every run
    // is retained. A genuine RED must not hide the remaining independent cases.
    for case in &matrix.cases {
        let project = capture.join("projects").join(case.project.as_str());
        std::fs::create_dir_all(&project).unwrap();
        let saved = capture.join("cases").join(case.id.as_str());
        std::fs::create_dir(&saved).unwrap();
        std::fs::create_dir(saved.join("inputs")).unwrap();
        for name in &matrix.input_files {
            let bytes = std::fs::read(fixture.join(case.id.as_str()).join(name.as_str())).unwrap();
            std::fs::write(project.join(name.as_str()), &bytes).unwrap();
            std::fs::write(saved.join("inputs").join(name.as_str()), &bytes).unwrap();
        }
        let project = project.canonicalize().unwrap();
        let before = input_manifest(&project, &matrix.input_files);
        write_json(&saved.join("inputs-before.json"), &before);
        let template =
            std::fs::read_to_string(fixture.join(case.id.as_str()).join("expected-report.json"))
                .unwrap();
        let expected = expected_stdout(&template, &project);
        std::fs::write(saved.join("expected.stdout.raw"), expected.as_bytes()).unwrap();
        let argv = [
            "check",
            "App.tsx",
            "--quiet",
            "--format",
            "json",
            "--corsa-path",
        ];
        let mut command = Command::new(&cli);
        command.current_dir(&project).args(argv).arg(&native);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut pid = None;
        let output = command
            .spawn()
            .map_err(|error| cstr!("{error}"))
            .and_then(|child| {
                pid = Some(child.id());
                child.wait_with_output().map_err(|error| cstr!("{error}"))
            });
        let after = input_manifest(&project, &matrix.input_files);
        write_json(&saved.join("inputs-after.json"), &after);
        let mut record = json!({
            "id":case.id,"repairOf":case.repair_of,"cwd":project,"pid":pid,
            "argv":[cli,"check","App.tsx","--quiet","--format","json","--corsa-path",native],
            "authority":authority,"inputBefore":before,"inputAfter":after,
            "expectedStdoutSha256":digest(expected.as_bytes()),
            "fixtureExpectedSha256":digest(template.as_bytes())
        });
        match &output {
            Ok(output) => {
                std::fs::write(saved.join("stdout.raw"), &output.stdout).unwrap();
                std::fs::write(saved.join("stderr.raw"), &output.stderr).unwrap();
                record["exitCode"] = json!(output.status.code());
                record["signal"] = json!(signal(output));
                record["processSuccess"] = json!(output.status.success());
                record["runError"] = Value::Null;
                record["stdoutBytes"] = json!(output.stdout.len());
                record["stderrBytes"] = json!(output.stderr.len());
                record["stdoutSha256"] = json!(digest(&output.stdout));
                record["stderrSha256"] = json!(digest(&output.stderr));
            }
            Err(error) => record["runError"] = json!(error),
        }
        write_json(&saved.join("process.json"), &record);
        records.push(record);
        runs.push(Run {
            id: case.id.clone(),
            expected_exit: case.exit_code,
            expected_stdout: expected,
            output,
            before,
            after,
        });
    }

    let mut failures = Vec::new();
    for run in &runs {
        let frozen = json!(frozen_inputs["files"].as_array().unwrap().iter()
            .filter(|file| file["case"] == run.id.as_str())
            .map(|file| json!({"file":file["file"],"bytes":file["bytes"],"sha256":file["sha256"]}))
            .collect::<Vec<_>>());
        if run.before != frozen || run.after != run.before {
            failures.push(cstr!("{}: authored input bytes changed", run.id));
        }
        match &run.output {
            Ok(output) => {
                if output.status.code() != Some(run.expected_exit) || signal(output).is_some() {
                    failures.push(cstr!(
                        "{}: unexpected process exit {:?}",
                        run.id,
                        output.status
                    ));
                }
                if !output.stderr.is_empty() {
                    failures.push(cstr!("{}: stderr is not empty", run.id));
                }
                if output.stdout != run.expected_stdout.as_bytes() {
                    failures.push(cstr!("{}: complete literal stdout differs", run.id));
                }
                let expected: Value = serde_json::from_str(&run.expected_stdout).unwrap();
                if serde_json::from_slice::<Value>(&output.stdout).ok() != Some(expected) {
                    failures.push(cstr!("{}: complete report differs", run.id));
                }
            }
            Err(error) => failures.push(cstr!("{}: CLI process failed: {error}", run.id)),
        }
    }
    if file_digest(&cli).as_str() != authority["sourceCliSha256"]
        || file_digest(&native).as_str() != authority["nativeSha256"]
        || file_digest(&collector).as_str() != authority["collectorSha256"]
        || source_sha(&workspace) != sha
    {
        failures.push("source/runtime authority changed during capture".into());
    }
    write_json(
        &capture.join("receipt.json"),
        &json!({
            "success":failures.is_empty(),"complete":true,"authority":authority,
            "scope":"eight cold source CLI packets; no persistent session or installed claim",
            "runs":records,"failures":failures
        }),
    );
    assert!(
        failures.is_empty(),
        "whole matrix failed; raw capture: {}\n{failures:#?}",
        capture.display()
    );
}
