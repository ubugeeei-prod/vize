//! Whole public config selection and native diagnostics with UTF-8 BOM inputs.
#![expect(clippy::disallowed_types, reason = "whole JSON fixture packets")]
#![expect(clippy::disallowed_methods, reason = "whole JSON fixture packets")]

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
    "/../../tests/_fixtures/differential/typechecker/tsconfig-bom-3984/input.json"
));
const EXPECTED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-bom-3984/expected-cli.json"
));
const STOCK: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-bom-3984/official-whole-observations.json"
));
const MALFORMED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-bom-3984/malformed-original-whole-controls.json"
));

fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn project(group: &str, corpus: &Value) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/vize-tests/tests")
        .join(cstr!("config-bom-{group}-{}", std::process::id()).as_str());
    assert!(!root.exists());
    for (name, text) in corpus["commonFiles"].as_object().unwrap() {
        write(&root, name, text.as_str().unwrap());
    }
    std::fs::canonicalize(root).unwrap()
}

fn packet(command: &mut Command) -> Value {
    let output = command.output().unwrap();
    json!({"exitCode":output.status.code(),
        "stdout":std::str::from_utf8(&output.stdout).unwrap(),
        "stderr":std::str::from_utf8(&output.stderr).unwrap()})
}

fn expected(root: &Path, phase: &str) -> Value {
    let mut whole: Value = serde_json::from_str(EXPECTED).unwrap();
    // The complete authored DTO binds only the known config-relative alias.
    // The producer's current response never supplies expected fields or paths.
    whole[phase]["programs"][0]["compilerOptions"]["paths"]["@model"][0] =
        json!(root.join("src/model.ts").to_str().unwrap());
    whole[phase].clone()
}

fn check(root: &Path, native: &Path, corpus: &Value, explicit: bool) -> Value {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .env("CORSA_PATH", native)
        .args(["check", "--quiet", "--format", "json"]);
    if explicit {
        for path in corpus["authoredSelectedFiles"].as_array().unwrap() {
            command.arg(path.as_str().unwrap());
        }
    }
    packet(&mut command)
}

fn run_group(group: &str, variants: &[&str], native: &Path) {
    let corpus: Value = serde_json::from_str(INPUT).unwrap();
    let stock: Value = serde_json::from_str(STOCK).unwrap();
    assert_eq!(
        packet(Command::new(native).arg("--version")),
        json!({"exitCode":0,"stdout":"Version 7.0.2\n","stderr":""})
    );
    let root = project(group, &corpus);
    let mut calls = 0;
    let mut phases = Vec::new();
    for phase in ["clean", "broken", "repair"] {
        write(
            &root,
            "src/main.ts",
            corpus["phases"][phase].as_str().unwrap(),
        );
        for &variant in variants {
            for (name, text) in corpus["cases"][group][variant].as_object().unwrap() {
                write(&root, name, text.as_str().unwrap());
            }
            let original = packet(Command::new(native).current_dir(&root).args([
                "--project",
                "tsconfig.json",
                "--pretty",
                "false",
            ]));
            assert_eq!(
                original,
                stock["records"][group][phase][variant]["official"]["typescript-7.0.2"]["check"],
                "stock {group}/{phase}/{variant}"
            );
            let mut actual = Vec::new();
            for explicit in [false, true] {
                let output = check(&root, native, &corpus, explicit);
                assert_eq!(
                    output["exitCode"],
                    json!(i32::from(phase == "broken")),
                    "{output}"
                );
                assert_eq!(output["stderr"], "", "{output}");
                let complete: Value =
                    serde_json::from_str(output["stdout"].as_str().unwrap()).unwrap();
                assert_eq!(
                    complete,
                    expected(&root, phase),
                    "{group}/{phase}/{variant}/explicit={explicit}"
                );
                calls += 1;
                actual.push(json!({"explicit":explicit,"wholePacket":output}));
            }
            phases.push(json!({"phase":phase,"variant":variant,"stock":original,"vize":actual}));
        }
    }
    assert_eq!(calls, variants.len() * 6);
    if let Some(capture) = std::env::var_os("VIZE_TSCONFIG_TYPES_CAPTURE") {
        let dir = PathBuf::from(capture).join("utf8-bom").join(group);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("whole-input.json"), INPUT).unwrap();
        std::fs::write(
            dir.join("runtime.json"),
            serde_json::to_vec_pretty(
                &json!({"nativeBinary":native,"cliBinary":env!("CARGO_BIN_EXE_vize"),
                "projectRoot":root,"phases":phases}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
    eprintln!("complete config BOM {group}: {calls} public native CLI invocations accepted");
}

#[test]
fn bom_root_config_keeps_whole_alias_globals_selection_and_diagnostics() {
    let Some(native) = corsa_requirement::required_or_skip(None::<PathBuf>) else {
        return;
    };
    run_group("direct", &["plain", "bom"], &native);
}

#[test]
fn bom_extended_root_or_parent_keeps_whole_authored_program_and_repair() {
    let Some(native) = corsa_requirement::required_or_skip(None::<PathBuf>) else {
        return;
    };
    run_group("extended", &["plain", "bom-root", "bom-parent"], &native);
}

#[test]
fn malformed_plain_and_bom_configs_preserve_whole_original_cli_failure() {
    let Some(native) = corsa_requirement::required_or_skip(None::<PathBuf>) else {
        return;
    };
    let corpus: Value = serde_json::from_str(INPUT).unwrap();
    let malformed: Value = serde_json::from_str(MALFORMED).unwrap();
    let mut calls = 0;
    let mut actual = Vec::new();
    for (group, variants) in malformed["wholeCases"].as_object().unwrap() {
        let root = project(cstr!("malformed-{group}").as_str(), &corpus);
        for (variant, files) in variants.as_object().unwrap() {
            for (name, text) in files.as_object().unwrap() {
                write(&root, name, text.as_str().unwrap());
            }
            for explicit in [false, true] {
                let output = check(&root, &native, &corpus, explicit);
                assert_eq!(
                    output, malformed["expectedPlainFailure"],
                    "{group}/{variant}/explicit={explicit}"
                );
                calls += 1;
                actual.push(json!({"group":group,"variant":variant,"explicit":explicit,"wholePacket":output}));
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    assert_eq!(calls, 8);
    if let Some(capture) = std::env::var_os("VIZE_TSCONFIG_TYPES_CAPTURE") {
        let dir = PathBuf::from(capture).join("utf8-bom");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("malformed-runtime.json"),
            serde_json::to_vec_pretty(&actual).unwrap(),
        )
        .unwrap();
    }
    eprintln!("complete config BOM malformed: {calls} public CLI failures accepted");
}
