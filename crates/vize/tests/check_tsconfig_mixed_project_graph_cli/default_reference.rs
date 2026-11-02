//! Later implicit project inputs must not depend on earlier program roots.
use super::{ProjectCase, resolve_test_corsa_path};
use serde_json::{Value, json};
use std::{fs, process::Command};

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-reference-defaults-3984/input.json"
));

#[test]
fn later_default_reference_checks_vue_template_and_repair() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project = ProjectCase::new("later-default-reference-3984");
    let input: Value = serde_json::from_str(INPUT).unwrap();
    for (name, body) in input["files"].as_object().unwrap() {
        project.write(name, body.as_str().unwrap());
    }
    project.write(
        "vite.config.mjs",
        &crate::support::vite_config::source(r#"{ "typeChecker": { "jsxTypecheck": true } }"#),
    );
    let original = input["files"]["beta/src/App.vue"].as_str().unwrap();
    let broken = check(&project, &corsa_path, "original");
    assert_programs(&broken);
    assert_eq!(broken["errorCount"], 1);
    assert_eq!(
        complete_diagnostics(&broken),
        vec![(
            "beta/src/App.vue".to_owned(),
            "error:6:20 [TS2339] Property 'missing' does not exist on type 'string'.".to_owned(),
        )]
    );
    let repaired = original.replace("title.missing", "title");
    assert_ne!(original, repaired);
    project.write("beta/src/App.vue", &repaired);
    let repair = check(&project, &corsa_path, "repair");
    assert_programs(&repair);
    assert_eq!(repair["errorCount"], 0);
    assert_eq!(complete_diagnostics(&repair), vec![]);
    project.write("beta/src/App.vue", original);
    let restored = check(&project, &corsa_path, "restore");
    assert_programs(&restored);
    assert_eq!(restored["errorCount"], 1);
    assert_eq!(
        complete_diagnostics(&restored),
        complete_diagnostics(&broken)
    );
}

fn check(project: &ProjectCase, corsa_path: &str, state: &str) -> Value {
    let program = env!("CARGO_BIN_EXE_vize");
    let args = ["check", "--format", "json"];
    let result = Command::new(program)
        .current_dir(&project.root)
        .env("CORSA_PATH", corsa_path)
        .args(args)
        .output();
    let output = match result {
        Ok(output) => output,
        Err(error) => {
            retain(
                state,
                &json!({"state":state,"program":program,"args":args,
                "cwd":project.root,"selectedCorsaPath":corsa_path,"spawnError":error.to_string()}),
            );
            panic!("failed to execute actual CLI: {error}");
        }
    };
    #[cfg(unix)]
    let signal = {
        use std::os::unix::process::ExitStatusExt;
        output.status.signal()
    };
    #[cfg(not(unix))]
    let signal: Option<i32> = None;
    retain(
        state,
        &json!({"state":state,"program":program,"args":args,
        "cwd":project.root,"selectedCorsaPath":corsa_path,"status":output.status.code(),
        "signal":signal,"spawnError":null,"stdoutBytes":output.stdout,"stderrBytes":output.stderr,
        "inputs":INPUT,"currentApp":fs::read(project.root.join("beta/src/App.vue")).unwrap()}),
    );
    assert_eq!(signal, None);
    assert_eq!(output.stderr.as_slice(), b"");
    assert_eq!(
        output.status.code(),
        Some(if state == "repair" { 0 } else { 1 })
    );
    serde_json::from_slice(&output.stdout).expect("the complete CLI output must be JSON")
}

fn retain(state: &str, packet: &Value) {
    let bytes = serde_json::to_vec_pretty(packet).unwrap();
    println!(
        "tsconfig-reference-defaults-3984 {state}: {}",
        String::from_utf8_lossy(&bytes)
    );
    if let Some(root) = std::env::var_os("VIZE_TSCONFIG_REFERENCE_CAPTURE") {
        let root = std::path::PathBuf::from(root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join(format!("{state}.json")), bytes).unwrap();
    }
}

fn assert_programs(packet: &Value) {
    let actual = packet["programs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|program| (program["tsconfig"].clone(), program["files"].clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            (json!("alpha/tsconfig.json"), json!(["alpha/src/owned.ts"])),
            (
                json!("beta/tsconfig.json"),
                json!([
                    "beta/src/App.vue",
                    "beta/src/contracts.ts",
                    "beta/src/entry.tsx"
                ])
            ),
        ]
    );
    assert_eq!(packet["fileCount"], 4);
}

fn complete_diagnostics(packet: &Value) -> Vec<(String, String)> {
    packet["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap();
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |diagnostic| (name.to_owned(), diagnostic.as_str().unwrap().to_owned()))
        })
        .collect()
}
