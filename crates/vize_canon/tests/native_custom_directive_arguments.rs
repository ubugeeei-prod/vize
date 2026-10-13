//! Whole native diagnostics for the pinned Element Plus dynamic-argument shape.
#![cfg(unix)]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "complete native fixture packets use std strings"
)]

use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use vize_canon::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/dynamic-custom-directive-argument/"
);
const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/dynamic-custom-directive-argument/App.vue.txt"
));
const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/dynamic-custom-directive-argument/tsconfig.json.txt"
));

fn native() -> Option<PathBuf> {
    let required = std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref() == Ok("1");
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        assert_eq!(
            required, false,
            "required custom directive laws cannot be disabled"
        );
        return None;
    }
    Some(
        vize_carton::corsa_resolver::resolve_corsa_executable(
            vize_carton::corsa_resolver::CorsaResolveRequest {
                explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
                project_root: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
            },
        )
        .expect("genuine native runtime is required")
        .canonicalize()
        .unwrap(),
    )
}

fn check(native: &Path, name: &str, source: &str) -> Vec<String> {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::write(root.join("tsconfig.json"), CONFIG).unwrap();
    fs::write(root.join("App.vue"), source).unwrap();
    fs::create_dir(root.join("node_modules")).unwrap();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    std::os::unix::fs::symlink(
        repo.join("tests/node_modules/vue").canonicalize().unwrap(),
        root.join("node_modules/vue"),
    )
    .unwrap();
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        root,
        BatchTypeCheckerOptions::default(),
        Some(native),
    )
    .unwrap();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let mut rows: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{}({},{}): severity {} TS{}: {}",
                diagnostic.file.file_name().unwrap().to_string_lossy(),
                diagnostic.line + 1,
                diagnostic.column + 1,
                diagnostic.severity,
                diagnostic.code.unwrap(),
                diagnostic.message,
            )
        })
        .collect();
    rows.sort();
    if let Some(destination) = std::env::var_os("VIZE_CUSTOM_DIRECTIVE_CAPTURE") {
        let directory = PathBuf::from(destination);
        fs::create_dir_all(&directory).unwrap();
        let generated: Vec<_> = checker
            .virtual_files()
            .iter()
            .map(|file| {
                serde_json::json!({
                    "originalPath": file.original_path,
                    "virtualPath": file.virtual_path,
                    "content": file.content.as_str(),
                })
            })
            .collect();
        fs::write(directory.join(format!("{name}.json")), serde_json::to_vec_pretty(&serde_json::json!({
            "native": native,
            "nativeSha256": Sha256::digest(fs::read(native).unwrap()).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "source": source,
            "configuration": CONFIG,
            "originalCustody": fs::read_to_string(format!("{CORPUS}original-custody.json")).unwrap(),
            "generated": generated,
            "sharedHelpers": checker.shared_helpers_preamble(),
            "diagnostics": rows,
        })).unwrap()).unwrap();
    }
    rows
}

#[test]
fn custom_directive_arguments_keep_native_expression_and_hook_value_errors() {
    let Some(native) = native() else {
        return;
    };
    let cases: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(format!("{CORPUS}cases.json")).unwrap()).unwrap();
    if std::env::var("VIZE_CUSTOM_DIRECTIVE_BASELINE").as_deref() == Ok("1") {
        let expected: Vec<String> =
            serde_json::from_value(cases["baselineDiagnostics"].clone()).unwrap();
        assert_eq!(check(&native, "baseline", ORIGINAL), expected);
        return;
    }
    for case in cases["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let mut source = ORIGINAL.to_owned();
        for edit in case["edits"].as_array().unwrap() {
            let from = edit["from"].as_str().unwrap();
            let to = edit["to"].as_str().unwrap();
            assert_eq!(edit["occurrences"], 1);
            source = source.replacen(from, to, 1);
        }
        let expected: Vec<String> = serde_json::from_value(case["diagnostics"].clone()).unwrap();
        assert_eq!(check(&native, name, &source), expected, "{name}");
        assert_eq!(
            check(&native, &format!("{name}-repaired"), ORIGINAL),
            Vec::<String>::new()
        );
    }
}
