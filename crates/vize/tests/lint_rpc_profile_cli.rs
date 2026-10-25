#![cfg(test)]

use crate::support::{corsa_path, corsa_requirement};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

#[test]
fn real_lint_rpc_phases_preserve_complete_diagnostics() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let Some(runtime) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace)) else {
        return;
    };
    let project = tempfile::tempdir().unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(project.path().join("package.json"), "{}").unwrap();
    fs::write(project.path().join("tsconfig.json"), r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","noEmit":true},"include":["src"]}"#).unwrap();
    fs::write(
        project.path().join("src/entry.ts"),
        "export async function save(): Promise<void> {}\nsave();\n",
    )
    .unwrap();
    let run = |profile| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command
            .current_dir(project.path())
            .env("CORSA_PATH", &runtime)
            .args([
                "lint",
                "--no-config",
                "--preset",
                "all",
                "--type-aware",
                "--format",
                "json",
                "src/entry.ts",
            ]);
        if profile {
            command.args(["--profile-json", "profile.json"]);
        }
        command.output().unwrap()
    };
    let plain = run(false);
    let profiled = run(true);
    assert_eq!(profiled.status.code(), plain.status.code());
    assert_eq!(
        profiled.stdout, plain.stdout,
        "profiling must preserve every diagnostic field"
    );
    let output: Value = serde_json::from_slice(&plain.stdout).unwrap();
    let files = output.as_array().unwrap();
    assert_eq!(files.len(), 1);
    let messages = files.first().unwrap()["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    let finding = messages.first().unwrap();
    assert_eq!(finding["ruleId"], "type/no-floating-promises");
    assert_eq!(
        json!([
            finding["line"],
            finding["column"],
            finding["endLine"],
            finding["endColumn"]
        ]),
        json!([2, 1, 2, 7])
    );
    let profile: Value =
        serde_json::from_slice(&fs::read(project.path().join("profile.json")).unwrap()).unwrap();
    assert_eq!(profile["schema_version"], 1);
    assert_eq!(
        profile["truncation"],
        json!({"dropped_spans":0,"dropped_counters":0})
    );
    let mut phases: Vec<_> = profile["spans"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["attribution"]["stage"] == "patina"
                && row["attribution"]["pass"] == "updateSnapshot"
        })
        .map(|row| {
            (
                row["key"].as_str().unwrap(),
                row["count"].as_u64().unwrap(),
                row["attribution"].clone(),
            )
        })
        .collect();
    phases.sort_by_key(|(key, _, _)| *key);
    let attribution = json!({"stage":"patina","pass":"updateSnapshot","block":"jsonrpc"});
    assert_eq!(
        phases,
        vec![
            ("corsa.request.deserialize_response", 2, attribution.clone()),
            ("corsa.request.serialize_params", 2, attribution.clone()),
            ("corsa.request.transport", 2, attribution),
        ]
    );
}
