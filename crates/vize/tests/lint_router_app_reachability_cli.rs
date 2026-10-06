//! Whole public CLI reports for #7932 and independently authored app boundaries.
//! Expectations come from the pinned diagnostic/serializer source, never recapture.
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "whole CLI source fixtures")]
#![expect(clippy::disallowed_types, reason = "whole CLI source fixtures")]

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const ARGS: &[&str] = &[
    "lint",
    "--cross-file",
    "src/**/*.vue",
    "src/**/*.ts",
    "--format",
    "json",
];

#[derive(Deserialize)]
struct Corpus {
    schema: u32,
    issue: String,
    reporter_database_id: u64,
    original_body_sha256: String,
    original_primary_sha256: String,
    original_audit_sha256: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    decision: String,
    inputs: Vec<Input>,
    args: Vec<String>,
    expected_stdout: String,
    expected_stderr: String,
    expected_exit: i32,
}

#[derive(Deserialize)]
struct Input {
    source: String,
    target: String,
    sha256: String,
    provenance: String,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/lint-regressions/router-app-reachability-7932")
}

fn corpus() -> Corpus {
    serde_json::from_slice(&fs::read(root().join("cases.json")).expect("read corpus"))
        .expect("parse corpus")
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn fence<'a>(body: &'a str, language: &str) -> &'a str {
    body.split_once(&format!("```{language}\n"))
        .expect("original fence")
        .1
        .split_once("```\n")
        .expect("fence terminator")
        .0
}

#[test]
fn original_public_sources_and_command_are_literal() {
    let root = root();
    let corpus = corpus();
    assert_eq!(corpus.schema, 1);
    assert_eq!(
        corpus.issue,
        "https://github.com/ubugeeei-prod/vize/issues/7932"
    );
    assert_eq!(corpus.reporter_database_id, 71201308);
    let primary_bytes = fs::read(root.join("original/issue.json")).expect("primary issue");
    let audit_bytes =
        fs::read(root.join("original/authenticated-issue.json")).expect("authenticated issue");
    assert_eq!(hash(&primary_bytes), corpus.original_primary_sha256);
    assert_eq!(hash(&audit_bytes), corpus.original_audit_sha256);
    let primary: Value = serde_json::from_slice(&primary_bytes).expect("primary JSON");
    let audit: Value = serde_json::from_slice(&audit_bytes).expect("authenticated JSON");
    let body = fs::read_to_string(root.join("original/issue.md")).expect("literal body");
    assert_eq!(primary["body"], body);
    assert_eq!(audit["body"], body);
    assert_eq!(audit["author"]["databaseId"], 71201308);
    assert_eq!(hash(body.as_bytes()), corpus.original_body_sha256);
    for (path, language) in [
        ("original/project/src/components/user-link.vue", "vue"),
        ("original/project/src/components/user-link.test.ts", "ts"),
        ("original/command.sh", "sh"),
    ] {
        assert_eq!(
            fs::read(root.join(path)).expect("original source"),
            fence(&body, language).as_bytes(),
            "literal reporter payload: {path}"
        );
    }
    assert_eq!(
        fence(&body, "sh"),
        "vize lint --cross-file \"src/**/*.vue\" \"src/**/*.ts\"\n"
    );
    let inline = fence(&body, "")
        .lines()
        .find_map(|line| line.strip_prefix("nuxt.config.ts            "))
        .expect("reporter inline config");
    assert_eq!(
        fs::read_to_string(root.join("original/project/nuxt.config.ts"))
            .expect("inline config carrier"),
        format!("{inline}\n")
    );
}

#[test]
fn cross_file_router_application_reports_are_complete() {
    let corpus = corpus();
    let names: Vec<_> = corpus.cases.iter().map(|case| case.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "original-nuxt-throwaway",
            "installed-app-reachability",
            "direct-router-controls",
            "absent-app-install",
            "dynamic-app-install",
            "conditional-app-install",
            "ambiguous-app-install",
            "shadowed-app-install",
            "shadowed-app-constructor",
            "dynamic-app-root",
            "logical-app-install",
            "factory-app-install",
            "ssr-memory-installed",
            "two-independent-apps",
            "independent-project-alpha",
            "independent-project-beta",
            "shared-component-two-apps",
            "arrow-default-parameter-install",
            "function-default-parameter-install",
            "class-field-app-install",
        ]
    );
    let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".into());
    let capture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile)
        .join("router-app-reachability-7932");
    fs::create_dir_all(&capture).expect("persistent whole observations");
    let mut failures = Vec::new();
    for case in corpus.cases {
        failures.extend(check_case(&case, &capture.join(&case.name)));
    }
    assert!(
        failures.is_empty(),
        "all cases recorded at {}:\n{}",
        capture.display(),
        failures.join("\n")
    );
}

fn check_case(case: &Case, capture: &Path) -> Vec<String> {
    let args: Vec<_> = case.args.iter().map(String::as_str).collect();
    assert_eq!(args, ARGS, "{}: original argument scope", case.name);
    let root = root();
    let dir = tempfile::tempdir().expect("isolated project");
    let project = dir.path().join("project");
    fs::create_dir(&project).expect("project root");
    let mut before_inputs = Vec::new();
    for input in &case.inputs {
        let source = fs::read(root.join(&input.source)).expect("whole authored source");
        assert_eq!(hash(&source), input.sha256, "{}", input.source);
        assert!(!input.provenance.is_empty(), "{}", input.source);
        let target = project.join(&input.target);
        fs::create_dir_all(target.parent().expect("source parent")).expect("source directories");
        fs::write(&target, &source).expect("stage whole source");
        before_inputs.push((
            input.target.clone(),
            Some(fs::read(&target).expect("whole inputs before spawn")),
        ));
    }
    let expected_bytes = fs::read(root.join(&case.expected_stdout)).expect("authored whole report");
    let expected: Value = serde_json::from_slice(&expected_bytes).expect("authored report JSON");
    let reported_files: Vec<_> = expected
        .as_array()
        .expect("whole file report")
        .iter()
        .map(|file| file["file"].as_str().expect("reported filename"))
        .collect();
    let mut linted_files: Vec<_> = case
        .inputs
        .iter()
        .map(|input| input.target.as_str())
        .filter(|path| {
            path.starts_with("src/") && (path.ends_with(".vue") || path.ends_with(".ts"))
        })
        .collect();
    linted_files.sort_unstable();
    assert_eq!(
        reported_files, linted_files,
        "{}: whole inventory",
        case.name
    );
    fs::create_dir_all(capture).expect("case observations");
    fs::write(capture.join("expected.stdout.bin"), &expected_bytes).expect("whole expectation");
    fs::write(capture.join("expected.stderr.bin"), &case.expected_stderr)
        .expect("stderr expectation");

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project)
        .args(&case.args)
        .output();
    let spawn_error = output.as_ref().err().map(ToString::to_string);
    let (status, stdout, stderr) = match output {
        Ok(output) => (output.status.code(), output.stdout, output.stderr),
        Err(_) => (None, Vec::new(), Vec::new()),
    };
    // Save raw streams before interpreting a diagnostic or asserting any result.
    fs::write(capture.join("observed.stdout.bin"), &stdout).expect("raw stdout");
    fs::write(capture.join("observed.stderr.bin"), &stderr).expect("raw stderr");
    let after_inputs: Vec<_> = case
        .inputs
        .iter()
        .map(|input| {
            (
                input.target.clone(),
                fs::read(project.join(&input.target)).ok(),
            )
        })
        .collect();
    let actual = serde_json::from_slice::<Value>(&stdout);
    let checks = [
        ("authored inputs unchanged", before_inputs == after_inputs),
        ("exit status", status == Some(case.expected_exit)),
        ("whole stderr", stderr == case.expected_stderr.as_bytes()),
        ("whole JSON vector", actual.as_ref().ok() == Some(&expected)),
        ("byte-exact stdout", stdout == expected_bytes),
    ];
    let receipt = serde_json::json!({
        "case": case.name, "decision": case.decision, "args": case.args,
        "status": status, "spawnError": spawn_error, "beforeInputs": before_inputs,
        "afterInputs": after_inputs, "wholeExpected": expected,
        "wholeCurrent": actual.as_ref().ok(),
        "parseError": actual.as_ref().err().map(ToString::to_string),
        "expectedStatus": case.expected_exit, "expectedStderr": case.expected_stderr,
        "checks": checks
    });
    fs::write(
        capture.join("process.json"),
        serde_json::to_vec_pretty(&receipt).expect("whole process receipt"),
    )
    .expect("save receipt before assertions");
    let raw_stdout = String::from_utf8_lossy(&stdout);
    let raw_stderr = String::from_utf8_lossy(&stderr);
    let context = format!(
        "{}: {}\nstatus: {:?}\nspawn error: {:?}\nwhole stdout:\n{}\nwhole stderr:\n{}",
        case.name, case.decision, status, spawn_error, raw_stdout, raw_stderr
    );
    checks
        .into_iter()
        .filter(|(_, passed)| !*passed)
        .map(|(check, _)| {
            format!(
                "{context}\nfailed {check}; whole receipt: {}",
                capture.display()
            )
        })
        .collect()
}
