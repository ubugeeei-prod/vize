//! Exercise long prop keys through the real mapper host, including negatives.

use crate::support::corsa_requirement;
use std::{path::PathBuf, process::Command};
use vize_l0::cstr;

use super::{
    TSGO_ENV, assert_success, check_project, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

#[test]
fn long_prop_names_with_listeners_preserve_complete_authored_diagnostics() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper long props: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name an executable");
    let project = project(true);

    let invalid = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_eq!(invalid.status.code(), Some(2), "{}", output_text(&invalid));
    assert!(invalid.stderr.is_empty(), "{}", output_text(&invalid));
    let lines = std::str::from_utf8(&invalid.stdout)
        .unwrap()
        .lines()
        .map(|line| line.replace('\\', "/"))
        .collect::<Vec<_>>();
    assert_eq!(
        lines,
        [
            "src/Invalid.vue(7,11): error TS2322: Type 'string' is not assignable to type 'number'.",
            "src/Invalid.vue(8,16): error TS2322: Type 'string' is not assignable to type 'number'.",
            "src/Invalid.vue(9,91): error TS2339: Property 'missing' does not exist on type 'number'.",
        ]
    );
    std::fs::remove_file(project.path().join("src/Invalid.vue")).unwrap();
    let valid = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_success(&valid);
    assert!(valid.stdout.is_empty(), "{}", output_text(&valid));
    assert!(valid.stderr.is_empty(), "{}", output_text(&valid));
}

fn project(mapper: bool) -> tempfile::TempDir {
    let fixture = workspace_root().join("tests/_fixtures/long-prop-kebab-case-10423");
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    for name in ["Child.vue", "App.vue", "Invalid.vue"] {
        std::fs::copy(
            fixture.join("src").join(cstr!("{name}.txt").as_str()),
            project.path().join("src").join(name),
        )
        .unwrap();
    }
    let mut config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.join("tsconfig.json.txt")).unwrap()).unwrap();
    if mapper {
        config["contentMappers"] =
            serde_json::json!([{ "package": "vize", "extensions": [".vue"] }]);
    }
    std::fs::write(
        project.path().join("tsconfig.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    if mapper {
        install_mapper_manifest(project.path());
    }
    install_vue_package(project.path());

    project
}

#[test]
fn native_cli_long_prop_names_keep_all_authored_findings() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let project = project(false);
    let check = || {
        Command::new(env!("CARGO_BIN_EXE_vize"))
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
            .unwrap()
    };
    let invalid = check();
    assert_eq!(invalid.status.code(), Some(1), "{}", output_text(&invalid));
    let result: serde_json::Value = serde_json::from_slice(&invalid.stdout).unwrap();
    assert_eq!(result["errorCount"], 3);
    assert_eq!(result["warningCount"], 0);
    assert_eq!(
        result["files"],
        serde_json::json!([
            {"file":"src/App.vue", "diagnostics":[]},
            {"file":"src/Child.vue", "diagnostics":[]},
            {"file":"src/Invalid.vue", "diagnostics":[
                "error:7:11 [TS2322] Type 'string' is not assignable to type 'number'. (source: <Child :selected-guidance-juku-test-template-parent-folder-id=\"'wrong'\" @close=\"() => {}\" />; binding: 'selected-guidance-juku-test-template-parent-folder-id')",
                "error:8:16 [TS2322] Type 'string' is not assignable to type 'number'. (source: <AsyncChild :selected-guidance-juku-test-template-parent-folder-id=\"'wrong'\" @close=\"() => {}\" />; binding: 'selected-guidance-juku-test-template-parent-folder-id')",
                "error:9:91 [TS2339] Property 'missing' does not exist on type 'number'. (source: <Child :selected-guidance-juku-test-template-parent-folder-id=\"1\" @save=\"value => value.missing\" />; binding: @save)"
            ]}
        ])
    );
    std::fs::remove_file(project.path().join("src/Invalid.vue")).unwrap();
    let valid = check();
    assert_success(&valid);
    let result: serde_json::Value = serde_json::from_slice(&valid.stdout).unwrap();
    assert_eq!(result["errorCount"], 0);
    assert_eq!(result["warningCount"], 0);
    assert_eq!(
        result["files"],
        serde_json::json!([
            {"file":"src/App.vue", "diagnostics":[]},
            {"file":"src/Child.vue", "diagnostics":[]}
        ])
    );
}
