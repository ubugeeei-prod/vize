//! The Options API typechecker packs replayed through the Content Mapper.
//!
//! Each pinned case of these packs is checked by `vize check` in
//! `fix_history_diagnostics`. Under `tsc --runExternalCode` the same inputs
//! must report the same diagnostics at the same authored positions, so a
//! change that only one route sees cannot pass unnoticed.

use super::{
    Path, PathBuf, TSGO_ENV, check_project, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};
use serde_json::{Value, json};

const PACKS: [&str; 1] = ["options-api-this-bridge-1366"];

/// `file(line,column): error TScode` for each pinned diagnostic.
fn expected_keys(case: &Value) -> Vec<std::string::String> {
    let mut keys: Vec<_> = case["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|diagnostic| {
            format!(
                "{}({},{}): error TS{}",
                diagnostic["file"].as_str().unwrap(),
                diagnostic["line"],
                diagnostic["column"],
                diagnostic["code"],
            )
        })
        .collect();
    keys.sort();
    keys
}

fn reported_keys(stdout: &str) -> Vec<std::string::String> {
    let mut keys: Vec<_> = stdout
        .lines()
        .filter_map(|line| {
            let (location, rest) = line.split_once("): error TS")?;
            let code: std::string::String =
                rest.chars().take_while(char::is_ascii_digit).collect();
            Some(format!("{}): error TS{code}", location.replace('\\', "/")))
        })
        .collect();
    keys.sort();
    keys
}

/// Write one case's inputs, naming the mapper in its `tsconfig.json`.
fn materialize(fixture: &Path, case: &Value, root: &Path) {
    for input in case["inputs"].as_array().unwrap() {
        let file = input["file"].as_str().unwrap();
        let source = std::fs::read(fixture.join(input["source"].as_str().unwrap())).unwrap();
        let destination = root.join(file);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        if file == "tsconfig.json" {
            let mut config: Value = serde_json::from_slice(&source).unwrap();
            config
                .as_object_mut()
                .unwrap()
                .entry("contentMappers")
                .or_insert_with(|| json!([{ "package": "vize", "extensions": [".vue"] }]));
            std::fs::write(destination, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        } else {
            std::fs::write(destination, source).unwrap();
        }
    }
}

#[test]
fn options_api_packs_report_the_batch_diagnostics_through_the_content_mapper() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping Options API pack replay: {TSGO_ENV} is not set");
        return;
    };
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    for pack in PACKS {
        let fixture = workspace_root()
            .join("tests/_fixtures/differential/typechecker")
            .join(pack);
        let cases: Value =
            serde_json::from_slice(&std::fs::read(fixture.join("cases.json")).unwrap()).unwrap();
        for case in cases["cases"].as_array().unwrap() {
            let project = tempfile::Builder::new()
                .prefix("options-api-pack-")
                .tempdir_in(&cases_root)
                .unwrap();
            install_mapper_manifest(project.path());
            install_vue_package(project.path());
            materialize(&fixture, case, project.path());
            let output = check_project(&tsgo, project.path(), "tsconfig.json");
            assert!(output.stderr.is_empty(), "{}", output_text(&output));
            assert_eq!(
                reported_keys(std::str::from_utf8(&output.stdout).unwrap()),
                expected_keys(case),
                "{pack}/{}\n{}",
                case["id"].as_str().unwrap(),
                output_text(&output)
            );
        }
    }
}
