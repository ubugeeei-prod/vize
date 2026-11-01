//! Original mixed Vue/private-package import diagnostic ownership report.
#![expect(
    clippy::disallowed_macros,
    reason = "fixtures retain complete process observations"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "fixture source strings use std methods"
)]
#![expect(
    clippy::disallowed_types,
    reason = "fixture observations use owned std strings"
)]

use std::{path::Path, process::Command};

use crate::support::{corsa_path, corsa_requirement};

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/mixed-vue-package-imports-11253/input.json"
));

#[derive(Clone, Copy, Debug)]
enum Case {
    Original,
    HashOnly,
    Relative,
    VuePackage,
    HashVue,
    MainOnly,
    Repair,
    RestoreMainOnly,
    RestoreOriginal,
}

fn source_files(input: &serde_json::Value, case: Case) -> Vec<(String, String)> {
    input["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            let name = file["path"].as_str().unwrap();
            let mut source = file["source"].as_str().unwrap().to_owned();
            if name == "src/main.ts" {
                match case {
                    Case::HashOnly => {
                        source = source
                            .replace("import App from \"./App.vue\";\n", "\n")
                            .replace("App, value, probe", "value, probe");
                    }
                    Case::Relative => source = source.replace("#src/plain", "./plain"),
                    Case::VuePackage => {
                        source = source.replace(
                            "import { value } from \"#src/plain\";",
                            "import { ref as value } from \"vue\";",
                        );
                    }
                    Case::HashVue => {
                        source = source
                            .replace("\"./App.vue\"", "\"#src/App.vue\"")
                            .replace("import { value } from \"#src/plain\";\n", "\n")
                            .replace("App, value, probe", "App, probe");
                    }
                    Case::Repair => source = source.replace("\"not a number\"", "1"),
                    _ => {}
                }
            }
            if name == "src/plain.ts"
                && matches!(case, Case::MainOnly | Case::Repair | Case::RestoreMainOnly)
            {
                source = source.replace("\"not a number\"", "1");
            }
            if name == "package.json" && matches!(case, Case::HashVue) {
                let mut manifest: serde_json::Value = serde_json::from_str(&source).unwrap();
                manifest["imports"]["#src/*.vue"] = "./src/*.vue".into();
                source = serde_json::to_string_pretty(&manifest).unwrap() + "\n";
            }
            (name.to_owned(), source)
        })
        .collect()
}

fn diagnostic(line: u32) -> String {
    format!(
        "error:{line}:7 [TS2322] Type 'string' is not assignable to type 'number'. (source: const probe: number = \"not a number\";)"
    )
}

#[test]
fn check_json_mixed_vue_and_private_imports_keep_every_authored_diagnostic() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let Some(corsa) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace)) else {
        return;
    };
    let input: serde_json::Value = serde_json::from_str(INPUT).unwrap();
    assert_eq!(input["schema"], "vize.mixed-vue-package-imports");
    assert_eq!(
        input["id"],
        "typechecker/regression/mixed-vue-package-imports-11253"
    );
    assert_eq!(input["files"].as_array().unwrap().len(), 5);
    let root = tempfile::tempdir().unwrap();
    let modules = workspace.join("tests/node_modules");
    let vue_manifest = std::fs::read_to_string(modules.join("vue/package.json"))
        .expect("the real test Vue provider must be installed; no fixture facade");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&modules, root.path().join("node_modules")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&modules, root.path().join("node_modules")).unwrap();
    let config: serde_json::Value = serde_json::from_str(
        input["files"].as_array().unwrap()[1]["source"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let mut outcomes = Vec::new();
    for case in [
        Case::Original,
        Case::HashOnly,
        Case::Relative,
        Case::VuePackage,
        Case::HashVue,
        Case::MainOnly,
        Case::Repair,
        Case::RestoreMainOnly,
        Case::RestoreOriginal,
    ] {
        for (name, source) in source_files(&input, case) {
            let path = root.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, source).unwrap();
        }
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root.path())
            .env("CORSA_PATH", &corsa)
            .args([
                "check",
                "--no-config",
                "--tsconfig",
                "tsconfig.json",
                "--format",
                "json",
                "--quiet",
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        let parsed = serde_json::from_str::<serde_json::Value>(&stdout);
        let main_errors = if matches!(case, Case::Repair) {
            Vec::new()
        } else {
            vec![diagnostic(4)]
        };
        let plain_errors = if matches!(case, Case::MainOnly | Case::Repair | Case::RestoreMainOnly)
        {
            Vec::new()
        } else {
            vec![diagnostic(3)]
        };
        let count = main_errors.len() + plain_errors.len();
        let expected = serde_json::json!({
            "files": [
                {"file":"src/App.vue", "diagnostics":[]},
                {"file":"src/main.ts", "diagnostics":main_errors},
                {"file":"src/plain.ts", "diagnostics":plain_errors}
            ],
            "programs":[{"root":".","tsconfig":"tsconfig.json","compilerOptions":config["compilerOptions"],"files":["src/App.vue","src/main.ts","src/plain.ts"]}],
            "errorCount":count,"warningCount":0,"fileCount":3
        });
        outcomes.push(serde_json::json!({
            "case":format!("{case:?}"), "status":output.status.code(),
            "stdout":stdout,"stderr":stderr,"parsed":parsed.ok(),
            "expected":expected,"expectedStatus":if count == 0 {0} else {1}
        }));
    }
    // All variants execute before any assertion, so the baseline failure keeps
    // the complete original/control/repair/restore process vectors together.
    eprintln!("11253 complete observations: {}", serde_json::to_string_pretty(&serde_json::json!({
        "input":input,"requestedCorsaPath":corsa,"testVueManifest":vue_manifest,
        "scope":"source CLI with existing test providers; not untouched public or WSL2 acceptance",
        "outcomes":outcomes
    })).unwrap());
    for outcome in &outcomes {
        assert_eq!(outcome["status"], outcome["expectedStatus"], "{outcome:#}");
        assert_eq!(outcome["stderr"], "", "{outcome:#}");
        assert_eq!(outcome["parsed"], outcome["expected"], "{outcome:#}");
    }
}
