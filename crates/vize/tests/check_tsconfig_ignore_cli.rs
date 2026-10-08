//! Complete configured-project packets with ignored authored source roots.
#![expect(clippy::disallowed_types, reason = "whole JSON corpus packets")]
#![expect(clippy::disallowed_methods, reason = "whole JSON corpus packets")]

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
    "/../../tests/_fixtures/differential/typechecker/tsconfig-ignore-3984/input.json"
));

fn write(root: &Path, files: &Value) {
    for (name, bytes) in files.as_object().unwrap() {
        let path = root.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes.as_str().unwrap()).unwrap();
    }
}

fn packet(command: &mut Command) -> Value {
    let output = command.output().unwrap();
    json!({"exitCode":output.status.code(),
        "stdout":std::str::from_utf8(&output.stdout).unwrap(),
        "stderr":std::str::from_utf8(&output.stderr).unwrap()})
}

fn expected(case: &Value, broken: bool) -> Value {
    let files = case["reportedFiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            let diagnostics = if broken {
                case["brokenDiagnostics"]
                    .get(file.as_str().unwrap())
                    .cloned()
                    .unwrap_or_else(|| json!([]))
            } else {
                json!([])
            };
            json!({"file":file,"diagnostics":diagnostics})
        })
        .collect::<Vec<_>>();
    json!({"files":files,"programs":[{"root":".","tsconfig":"tsconfig.json",
        "compilerOptions":case["compilerOptions"],"files":case["programFiles"]}],
        "errorCount":if broken {4} else {0},"warningCount":0,"fileCount":6})
}

fn run_case(name: &str, native: &Path) {
    let corpus: Value = serde_json::from_str(INPUT).unwrap();
    let case = &corpus["cases"][name];
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/vize-tests/tests")
        .join(cstr!("config-ignore-{name}-{}", std::process::id()).as_str());
    assert!(!root.exists());
    write(&root, &corpus["commonFiles"]);
    write(&root, &case["files"]);
    let root = std::fs::canonicalize(root).unwrap();
    assert_eq!(
        packet(Command::new(native).arg("--version")),
        json!({"exitCode":0,"stdout":"Version 7.0.2\n","stderr":""})
    );
    let config = packet(Command::new(native).current_dir(&root).args([
        "--project",
        "tsconfig.json",
        "--showConfig",
    ]));
    assert_eq!(
        config, case["officialShowConfig"]["typescript-7.0.2"],
        "whole {name} native config"
    );
    let mut phases = Vec::new();
    let mut calls = 0;
    for phase in ["clean", "broken", "repair"] {
        write(
            &root,
            if phase == "broken" {
                &corpus["brokenFiles"]
            } else {
                &corpus["commonFiles"]
            },
        );
        let stock = packet(Command::new(native).current_dir(&root).args([
            "--project",
            "tsconfig.json",
            "--pretty",
            "false",
        ]));
        assert_eq!(
            stock, case["officialWholeOutputs"]["typescript-7.0.2"][phase],
            "whole {name}/{phase} native diagnostics"
        );
        let mut modes = Vec::new();
        for explicit in [false, true] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
            command
                .current_dir(&root)
                .env("CORSA_PATH", native)
                .args(["check", "--quiet", "--format", "json"]);
            if explicit {
                for file in case["reportedFiles"].as_array().unwrap() {
                    command.arg(file.as_str().unwrap());
                }
            }
            let whole = packet(&mut command);
            assert_eq!(whole["exitCode"], i32::from(phase == "broken"), "{whole}");
            assert_eq!(whole["stderr"], "", "{whole}");
            let response: Value = serde_json::from_str(whole["stdout"].as_str().unwrap()).unwrap();
            assert_eq!(
                response,
                expected(case, phase == "broken"),
                "whole {name}/{phase}/explicit={explicit}"
            );
            calls += 1;
            modes.push(json!({"explicit":explicit,"wholePacket":whole}));
        }
        phases.push(json!({"phase":phase,"stock":stock,"vize":modes}));
    }
    assert_eq!(calls, 6);
    if let Some(capture) = std::env::var_os("VIZE_TSCONFIG_TYPES_CAPTURE") {
        let dir = PathBuf::from(capture).join("ignore-files").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("whole-input.json"), INPUT).unwrap();
        std::fs::write(
            dir.join("runtime.json"),
            serde_json::to_vec_pretty(&json!({
            "nativeBinary":native,"cliBinary":env!("CARGO_BIN_EXE_vize"),
            "projectRoot":root,"nativeShowConfig":config,"phases":phases}))
            .unwrap(),
        )
        .unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
    eprintln!("complete configured ignore {name}: {calls} public native CLI invocations accepted");
}

#[test]
fn configured_ignored_ts_vue_and_declaration_roots_keep_whole_diagnostics_and_repair() {
    let Some(native) = corsa_requirement::required_or_skip(None::<PathBuf>) else {
        return;
    };
    run_case("direct", &native);
}

#[test]
fn inherited_ignored_ts_vue_and_declaration_roots_keep_whole_diagnostics_and_repair() {
    let Some(native) = corsa_requirement::required_or_skip(None::<PathBuf>) else {
        return;
    };
    run_case("inherited", &native);
}
