//! Component navigation must not report on undeclared attributes (#12531).

use std::{path::PathBuf, process::Command};

use serde_json::{Value, json};
use vize_l0::cstr;

use super::{
    TSGO_ENV, assert_success, check_project, install_mapper_manifest, install_vue_package,
    output_text,
};

const CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/nav-index-signature-access-12531/Child.vue.txt"
));
const APP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/nav-index-signature-access-12531/App.vue.txt"
));
const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/nav-index-signature-access-12531/tsconfig.json.txt"
));
const UNDECLARED: &str = r#"aria-label="y" class="c" :tabindex="1""#;

#[test]
fn undeclared_attributes_stay_clean_under_no_property_access_from_index_signature() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping navigation index-signature access: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name the real host");
    // The issue repro, then a control with a wrong declared prop value.
    let wrong = APP.replace(UNDECLARED, r#":count="'wrong'""#);
    assert_ne!(wrong, APP);
    for (app, invalid) in [(APP, false), (wrong.as_str(), true)] {
        let template = app
            .lines()
            .find(|line| line.trim_start().starts_with("<Child"))
            .unwrap()
            .trim();
        let project = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(project.path().join("src")).unwrap();
        std::fs::create_dir_all(project.path().join("node_modules")).unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        std::fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
        std::fs::write(
            project.path().join("tsconfig.mapper.json"),
            serde_json::to_vec_pretty(&json!({
                "extends": "./tsconfig.json",
                "contentMappers": [{"package": "vize", "extensions": [".vue"]}]
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(project.path().join("src/Child.vue"), CHILD).unwrap();
        std::fs::write(project.path().join("src/App.vue"), app).unwrap();
        let cli = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .env("CORSA_PATH", &tsgo)
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
        let packet: Value = serde_json::from_slice(&cli.stdout).unwrap();
        let files = packet["files"].as_array().unwrap();
        let app = files
            .iter()
            .find(|file| file["file"] == "src/App.vue")
            .unwrap();
        let mapper = check_project(&tsgo, project.path(), "tsconfig.mapper.json");
        assert_eq!(mapper.stderr, b"", "{}", output_text(&mapper));
        let mapper_lines = std::str::from_utf8(&mapper.stdout)
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if invalid {
            assert_eq!(packet["errorCount"], 1, "{packet}");
            assert_eq!(
                app["diagnostics"],
                json!([cstr!(
                    "error:5:35 [TS2322] Type 'string' is not assignable to type 'number'. (source: {template}; binding: 'count')"
                )]),
                "{packet}"
            );
            assert_eq!(
                mapper_lines,
                [
                    "src/App.vue(5,35): error TS2322: Type 'string' is not assignable to type 'number'."
                ],
                "{}",
                output_text(&mapper)
            );
        } else {
            assert_eq!(packet["errorCount"], 0, "{packet}");
            assert_eq!(app["diagnostics"], json!([]), "{packet}");
            assert_success(&mapper);
            assert_eq!(
                mapper_lines,
                Vec::<String>::new(),
                "{}",
                output_text(&mapper)
            );
        }
    }
}
