//! Whole configured type-library selection with authored Vue and TS controls.
#![expect(
    clippy::disallowed_types,
    reason = "whole JSON corpus uses std strings"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "whole JSON corpus uses std strings"
)]

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use vize_l0::cstr;

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-types-extends-3984/input.json"
));

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

fn write(root: &Path, name: &str, content: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn create_project(name: &str, corpus: &Value, case: &Value) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = workspace_root()
        .join("target/vize-tests/tests")
        .join(cstr!("types-extends-{name}-{}-{id}", std::process::id()).as_str());
    assert!(!root.exists(), "unique fixture root should not exist");
    for group in [&corpus["commonFiles"], &case["files"]] {
        for (path, content) in group.as_object().unwrap() {
            write(&root, path, content.as_str().unwrap());
        }
    }
    root
}

fn expected(case: &Value, broken: bool) -> Value {
    let diagnostics = if broken {
        case["brokenDiagnostics"].clone()
    } else {
        json!([])
    };
    let files = case["reportedFiles"].as_array().unwrap().iter().map(|file| {
        json!({"file":file,"diagnostics":if file.as_str()==Some("src/main.ts") { diagnostics.clone() } else { json!([]) }})
    }).collect::<Vec<_>>();
    json!({"files":files,"programs":[{"root":".","tsconfig":"tsconfig.json",
        "compilerOptions":case["compilerOptions"],"files":case["programFiles"]}],
        "errorCount":diagnostics.as_array().unwrap().len(),"warningCount":0,"fileCount":2})
}

fn check(root: &Path, native: &Path, case: &Value, explicit: bool, broken: bool) -> Value {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .env("CORSA_PATH", native)
        .args(["check", "--quiet", "--format", "json"]);
    if explicit {
        for file in case["reportedFiles"].as_array().unwrap() {
            command.arg(file.as_str().unwrap());
        }
    }
    let output = command.output().unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(i32::from(broken)),
        "{stdout}\n{stderr}"
    );
    assert_eq!(stderr, "", "{stdout}");
    serde_json::from_str(stdout).unwrap_or_else(|error| panic!("{error}: {stdout}"))
}

fn stock_check(root: &Path, native: &Path, case: &Value, phase: &str) -> Value {
    let output = Command::new(native)
        .current_dir(root)
        .args(["--project", "tsconfig.json", "--pretty", "false"])
        .output()
        .unwrap();
    let packet = json!({"exitCode":output.status.code(),
        "stdout":std::str::from_utf8(&output.stdout).unwrap(),
        "stderr":std::str::from_utf8(&output.stderr).unwrap()});
    assert_eq!(
        packet, case["officialWholeOutputs"]["typescript-7.0.2"][phase],
        "stock {phase}"
    );
    packet
}

fn run_cases(corpus: &Value, names: &str, native: &Path) {
    let version = Command::new(native).arg("--version").output().unwrap();
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(
        std::str::from_utf8(&version.stdout).unwrap().trim(),
        "Version 7.0.2"
    );
    assert!(version.stderr.is_empty());
    let mut invocations = 0;
    for name in corpus[names].as_array().unwrap() {
        let name = name.as_str().unwrap();
        let case = &corpus["cases"][name];
        let root = create_project(name, corpus, case);
        let mut phases = Vec::new();
        for (phase, source, broken) in [
            (
                "clean",
                case["files"]["src/main.ts"].as_str().unwrap(),
                false,
            ),
            ("broken", case["brokenSource"].as_str().unwrap(), true),
            ("repair", case["repairSource"].as_str().unwrap(), false),
        ] {
            write(&root, "src/main.ts", source);
            let oracle = stock_check(&root, native, case, phase);
            let default = check(&root, native, case, false, broken);
            let explicit = check(&root, native, case, true, broken);
            invocations += 2;
            let whole = expected(case, broken);
            assert_eq!(default, whole, "default {name}/{phase}");
            assert_eq!(explicit, whole, "explicit {name}/{phase}");
            phases
                .push(json!({"phase":phase,"stock":oracle,"default":default,"explicit":explicit}));
        }
        // Preserve complete actual packets and original inputs for hosted receipts.
        if let Some(capture) = std::env::var_os("VIZE_TSCONFIG_TYPES_CAPTURE") {
            let dir = PathBuf::from(capture).join(names).join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("whole-corpus.json"), INPUT).unwrap();
            std::fs::write(
                dir.join("runtime.json"),
                serde_json::to_vec_pretty(&json!({
                "nativeBinary":native,"cliBinary":env!("CARGO_BIN_EXE_vize"),
                "projectRoot":root,"phases":phases}))
                .unwrap(),
            )
            .unwrap();
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    assert_eq!(invocations, corpus[names].as_array().unwrap().len() * 6);
    eprintln!("complete types-array {names}: {invocations} public native CLI invocations accepted");
}

#[test]
fn types_extends_selects_whole_globals_and_vue_component_props_default_and_explicit() {
    let Some(native) = corsa_requirement::required_or_skip(None::<PathBuf>) else {
        return;
    };
    run_cases(
        &serde_json::from_str(INPUT).unwrap(),
        "publicCases",
        &native,
    );
}

#[test]
fn empty_types_keeps_whole_explicit_imports_directives_and_included_declarations() {
    let Some(native) = corsa_requirement::required_or_skip(None::<PathBuf>) else {
        return;
    };
    run_cases(
        &serde_json::from_str(INPUT).unwrap(),
        "emptyPublicCases",
        &native,
    );
}
