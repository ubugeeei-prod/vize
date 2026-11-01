//! Original #11252 input plus an authored error/repair through mapper and CLI.

use std::{fs, path::PathBuf, process::Command};

use serde_json::json;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/runtime-null-prop-11252/Box.vue.txt"
));
const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/runtime-null-prop-11252/tsconfig.json.txt"
));
const PACKAGE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/runtime-null-prop-11252/package.json.txt"
));
const TYPED_ERROR: &str = r#"<script setup lang="ts">
const props = defineProps({ as: { type: null }, label: { type: String, required: true } });
const invalid: number = props.label;
</script>
<template><div /></template>
"#;

#[test]
fn runtime_null_prop_keeps_original_acceptance_and_authored_error_repair() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("runtime-null-prop-11252-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("package.json"), PACKAGE).unwrap();
    fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    fs::write(project.path().join("src/Box.vue"), ORIGINAL).unwrap();

    for (phase, typed) in [
        ("original", None),
        ("authored-error", Some(TYPED_ERROR.to_owned())),
        (
            "repair",
            Some(TYPED_ERROR.replace("invalid: number", "invalid: string")),
        ),
    ] {
        if let Some(typed) = &typed {
            fs::write(project.path().join("src/Typed.vue"), typed).unwrap();
        }
        let bad = phase == "authored-error";
        let mapper = check_project(&tsgo, project.path(), "tsconfig.json");
        // Retain complete process outcomes before any verdict, including failures.
        println!(
            "runtime-null-prop-observation {}",
            json!({"phase":phase,"host":"mapper","native":tsgo,"cwd":project.path(),
                "original":ORIGINAL,"typed":typed,"config":CONFIG,
                "status":mapper.status.code(),"stdout":mapper.stdout,"stderr":mapper.stderr})
        );
        assert_eq!(
            mapper.status.code(),
            Some(if bad { 2 } else { 0 }),
            "{}",
            output_text(&mapper)
        );
        let expected = if bad {
            "src/Typed.vue(3,7): error TS2322: Type 'string' is not assignable to type 'number'.\n"
        } else {
            ""
        };
        assert_eq!(
            mapper.stdout,
            expected.as_bytes(),
            "{}",
            output_text(&mapper)
        );
        assert_eq!(mapper.stderr, b"", "{}", output_text(&mapper));

        let cli = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .env("CORSA_PATH", &tsgo)
            .args([
                "check",
                "--quiet",
                "--format",
                "json",
                "--tsconfig",
                "tsconfig.json",
                "src",
            ])
            .output()
            .unwrap();
        println!(
            "runtime-null-prop-observation {}",
            json!({"phase":phase,"host":"cli","native":tsgo,"cwd":project.path(),
                "status":cli.status.code(),"stdout":cli.stdout,"stderr":cli.stderr})
        );
        assert_eq!(
            cli.status.code(),
            Some(i32::from(bad)),
            "{}",
            output_text(&cli)
        );
        assert_eq!(cli.stderr, b"", "{}", output_text(&cli));
        let mut files = vec![json!({"file":"src/Box.vue","diagnostics":[]})];
        let mut paths = vec!["src/Box.vue"];
        if typed.is_some() {
            paths.push("src/Typed.vue");
            let errors = if bad {
                vec!["error:3:7 [TS2322] Type 'string' is not assignable to type 'number'."]
            } else {
                Vec::new()
            };
            files.push(json!({"file":"src/Typed.vue","diagnostics":errors}));
        }
        let config: serde_json::Value = serde_json::from_str(CONFIG).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&cli.stdout).unwrap(),
            json!({"files":files,"programs":[{"root":".","tsconfig":"tsconfig.json",
                "compilerOptions":config["compilerOptions"],"files":paths}],
                "errorCount":usize::from(bad),"warningCount":0,"fileCount":paths.len()}),
            "{}",
            output_text(&cli)
        );
    }
}
