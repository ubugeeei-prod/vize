use std::{path::PathBuf, process::Command};

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};
use crate::support::corsa_requirement;

const REFUSAL: &str =
    "pug `include` is not supported in Vue templates: it links other pug files at build time";

fn project(mapper: bool) -> tempfile::TempDir {
    let fixture =
        workspace_root().join("tests/_fixtures/differential/typechecker/pug-refusal-10430");
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    std::fs::create_dir_all(project.path().join("node_modules")).unwrap();
    for (input, output) in [
        ("src/App.vue.txt", "src/App.vue"),
        ("src/Invalid.vue.txt", "src/Invalid.vue"),
        ("tsconfig.json.txt", "tsconfig.json"),
    ] {
        std::fs::copy(fixture.join(input), project.path().join(output)).unwrap();
    }
    if mapper {
        let config_path = project.path().join("tsconfig.json");
        let mut config: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&config_path).unwrap()).unwrap();
        config["contentMappers"] =
            serde_json::json!([{ "package": "vize", "extensions": [".vue"] }]);
        std::fs::write(config_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        install_mapper_manifest(project.path());
    }
    install_vue_package(project.path());
    project
}

fn phases(original: &str) -> [(vize_l0::String, bool, bool); 5] {
    let valid = original.replace("include other.pug", "p text");
    let broken_script = original.replace("= 1;", "= 'bad';");
    let repaired_template = broken_script.replace("include other.pug", "p text");
    [
        (valid.as_str().into(), false, false),
        (original.into(), true, false),
        (broken_script.as_str().into(), true, true),
        (repaired_template.as_str().into(), false, true),
        (valid.as_str().into(), false, false),
    ]
}

#[test]
fn real_mapper_retains_preprocessor_errors_and_independent_script_checks_through_repairs() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Pug Content Mapper host: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file());
    let project = project(true);
    let path = project.path().join("src/Invalid.vue");
    let original = std::fs::read_to_string(&path).unwrap();
    for (source, refused, script_error) in phases(&original) {
        std::fs::write(&path, source.as_bytes()).unwrap();
        let output = check_project(&tsgo, project.path(), "tsconfig.json");
        assert_eq!(
            output.status.code(),
            Some(if refused || script_error { 2 } else { 0 }),
            "{}",
            output_text(&output)
        );
        assert!(output.stderr.is_empty(), "{}", output_text(&output));
        let mut lines = std::str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .map(|line| line.replace('\\', "/"))
            .collect::<Vec<_>>();
        lines.sort();
        let mut expected = Vec::new();
        if refused {
            expected.push(vize_l0::cstr!(
                "src/Invalid.vue(3,1): error vize100001: {REFUSAL}"
            ));
        }
        // This host treats mapper diagnostics as syntax errors and stops the
        // semantic pass. The next edit must reveal the retained script error.
        if script_error && !refused {
            expected.push("src/Invalid.vue(6,7): error TS2322: Type 'string' is not assignable to type 'number'.".into());
        }
        expected.sort();
        assert_eq!(lines, expected);
    }
}

#[test]
fn native_cli_retains_preprocessor_errors_and_independent_script_checks_through_repairs() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let project = project(false);
    let path = project.path().join("src/Invalid.vue");
    let original = std::fs::read_to_string(&path).unwrap();
    for (source, refused, script_error) in phases(&original) {
        std::fs::write(&path, source.as_bytes()).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .env("CORSA_PATH", &corsa)
            .args([
                "check",
                "--no-config",
                "--tsconfig",
                "tsconfig.json",
                "--format",
                "json",
            ])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(i32::from(refused || script_error)),
            "{}",
            output_text(&output)
        );
        let expected_progress = vize_l0::cstr!(
            "Building Corsa virtual project for 2 files under {}...\nRunning Corsa diagnostics for 2 files...\n",
            std::fs::canonicalize(project.path()).unwrap().display()
        );
        assert_eq!(
            std::str::from_utf8(&output.stderr).unwrap(),
            expected_progress.as_str()
        );
        let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            result["errorCount"],
            usize::from(refused) + usize::from(script_error)
        );
        assert_eq!(result["warningCount"], 0);
        let mut expected = Vec::new();
        if refused {
            expected.push(vize_l0::cstr!(
                "error:3:1 {REFUSAL} (source: include other.pug)"
            ));
        }
        if script_error {
            expected.push(
                "error:6:7 [TS2322] Type 'string' is not assignable to type 'number'.".into(),
            );
        }
        expected.sort();
        let mut files = result["files"].as_array().unwrap().clone();
        for file in &mut files {
            file["diagnostics"]
                .as_array_mut()
                .unwrap()
                .sort_by(|left, right| left.as_str().cmp(&right.as_str()));
        }
        assert_eq!(
            files,
            serde_json::json!([
                {"file":"src/App.vue", "diagnostics":[]},
                {"file":"src/Invalid.vue", "diagnostics":expected},
            ])
            .as_array()
            .unwrap()
            .clone()
        );
    }
}
