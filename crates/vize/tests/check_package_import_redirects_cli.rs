//! Original imported-file diagnostics and versioned private package redirects.
#![expect(
    clippy::disallowed_macros,
    reason = "fixtures retain whole process observations"
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
    "/../../tests/_fixtures/differential/typechecker/package-import-redirects-11317/input.json"
));

#[derive(Clone, Copy, Debug)]
enum Case {
    Original,
    NoVersion,
    Relative,
    NoVue,
    VueImporter,
    Repair,
    RestoreOriginal,
    VueImporterRepair,
    VueImporterRestore,
}

impl Case {
    fn vue_importer(self) -> bool {
        matches!(
            self,
            Self::VueImporter | Self::VueImporterRepair | Self::VueImporterRestore
        )
    }

    fn repaired(self) -> bool {
        matches!(self, Self::Repair | Self::VueImporterRepair)
    }
}

fn source_files(input: &serde_json::Value, case: Case) -> Vec<(String, String)> {
    let mut files = input["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            let name = file["path"].as_str().unwrap();
            let mut source = file["source"].as_str().unwrap().to_owned();
            if name == "package.json" && matches!(case, Case::NoVersion) {
                source = source.replace("  \"version\": \"0.0.0\",\n", "");
            }
            if name == "src/main.ts" {
                if matches!(case, Case::Relative) {
                    source = source.replace("#src/plain", "./plain");
                } else if case.vue_importer() {
                    source = "export {};\n".to_owned();
                }
            }
            if name == "src/plain.ts" {
                if matches!(case, Case::NoVue) {
                    source = source
                        .replace("import Other from \"./Other.vue\";\n", "\n")
                        .replace("Other, probe", "probe");
                }
                if case.repaired() {
                    source = source.replace("\"not a number\"", "1");
                }
            }
            (name.to_owned(), source)
        })
        .collect::<Vec<_>>();
    if case.vue_importer() {
        files.push(("src/Importer.vue".to_owned(), "<script setup lang=\"ts\">\nimport { value } from \"#src/plain\";\nconst imported = value;\n</script>\n<template><p>{{ imported }}</p></template>\n".to_owned()));
    }
    files
}

#[test]
fn check_json_private_import_redirects_keep_imported_diagnostics() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let Some(corsa) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace)) else {
        return;
    };
    let input: serde_json::Value = serde_json::from_str(INPUT).unwrap();
    assert_eq!(input["schema"], "vize.package-import-redirects");
    assert_eq!(
        input["id"],
        "typechecker/regression/package-import-redirects-11317"
    );
    assert_eq!(input["files"].as_array().unwrap().len(), 5);
    let root = tempfile::tempdir().unwrap();
    let modules = workspace.join("tests/node_modules");
    let vue_manifest = std::fs::read_to_string(modules.join("vue/package.json"))
        .expect("the real test Vue provider is required; no fixture facade");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&modules, root.path().join("node_modules")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&modules, root.path().join("node_modules")).unwrap();
    let config: serde_json::Value =
        serde_json::from_str(input["files"][1]["source"].as_str().unwrap()).unwrap();
    let mut outcomes = Vec::new();
    for case in [
        Case::Original,
        Case::NoVersion,
        Case::Relative,
        Case::NoVue,
        Case::VueImporter,
        Case::Repair,
        Case::RestoreOriginal,
        Case::VueImporterRepair,
        Case::VueImporterRestore,
    ] {
        let extra = root.path().join("src/Importer.vue");
        if extra.is_file() {
            std::fs::remove_file(extra).unwrap();
        }
        let files = source_files(&input, case);
        for (name, source) in &files {
            let path = root.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, source).unwrap();
        }
        let diagnostics = if case.repaired() {
            Vec::new()
        } else {
            vec!["error:5:7 [TS2322] Type 'string' is not assignable to type 'number'."]
        };
        let mut paths = vec!["src/Other.vue", "src/main.ts", "src/plain.ts"];
        if case.vue_importer() {
            paths.push("src/Importer.vue");
        }
        paths.sort();
        let expected_files = paths.iter().map(|name| serde_json::json!({
            "file":name,"diagnostics":if *name == "src/plain.ts" {diagnostics.clone()} else {Vec::new()}
        })).collect::<Vec<_>>();
        let expected = serde_json::json!({
            "files":expected_files,
            "programs":[{"root":".","tsconfig":"tsconfig.json","compilerOptions":config["compilerOptions"],"files":paths}],
            "errorCount":diagnostics.len(),"warningCount":0,"fileCount":paths.len()
        });
        let mut record = serde_json::json!({
            "case":format!("{case:?}"),"files":files,"expected":expected,
            "expectedStatus":if case.repaired() {0} else {1},
            "status":null,"stdout":null,"stderr":null,"parsed":null
        });
        let result = Command::new(env!("CARGO_BIN_EXE_vize"))
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
            .output();
        match result {
            Ok(output) => {
                record["status"] = output.status.code().into();
                record["osStatus"] = format!("{}", output.status).into();
                let stdout = String::from_utf8(output.stdout.clone());
                let stderr = String::from_utf8(output.stderr.clone());
                record["stdoutBytes"] = output.stdout.into();
                record["stderrBytes"] = output.stderr.into();
                record["parsed"] = stdout
                    .as_ref()
                    .ok()
                    .and_then(|source| serde_json::from_str::<serde_json::Value>(source).ok())
                    .into();
                record["stdout"] = stdout.as_ref().ok().cloned().into();
                record["stderr"] = stderr.as_ref().ok().cloned().into();
                record["stdoutUtf8Error"] = stdout.err().map(|error| error.to_string()).into();
                record["stderrUtf8Error"] = stderr.err().map(|error| error.to_string()).into();
            }
            Err(error) => record["spawnError"] = error.to_string().into(),
        }
        outcomes.push(record);
    }
    // Even a spawn/UTF-8/semantic failure leaves every requested case visible.
    eprintln!("11317 complete observations: {}", serde_json::to_string_pretty(&serde_json::json!({
        "input":input,"requestedCorsaPath":corsa,"testVueManifest":vue_manifest,
        "scope":"source CLI and existing real test providers; not untouched public or WSL2 acceptance",
        "outcomes":outcomes
    })).unwrap());
    for outcome in &outcomes {
        assert_eq!(outcome["status"], outcome["expectedStatus"], "{outcome:#}");
        assert_eq!(outcome["stderr"], "", "{outcome:#}");
        assert_eq!(outcome["parsed"], outcome["expected"], "{outcome:#}");
    }
}
