//! Whole project selection and emitted-package consumers share the same fixture.
#![expect(
    clippy::disallowed_types,
    reason = "complete corpus and JSON use std strings"
)]
use super::*;
use serde_json::{Value, json};

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-diamond-3984/input.json"
));
const DECLARATION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-diamond-3984/main.d.ts.expected.txt"
));
const DECLARATION_MAP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-diamond-3984/main.d.ts.map.expected.json"
));

fn fixture() -> Value {
    serde_json::from_str(INPUT).unwrap()
}

fn create_library(name: &str, corpus: &Value) -> std::path::PathBuf {
    let entries = corpus["files"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, content)| (name.as_str(), content.as_str().unwrap()))
        .collect::<Vec<_>>();
    let root = create_cli_project(name, &entries);
    vue_stub::install_vue_jsx_type_stub(&root);
    root
}

fn check(root: &Path, corsa: &str, declaration: bool, expected_status: i32) -> Value {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .env("CORSA_PATH", corsa)
        .args(["check", "--format", "json"]);
    if declaration {
        command.arg("--declaration");
    }
    let output = command.output().unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(expected_status),
        "{stdout}\n{stderr}"
    );
    assert!(stderr.is_empty(), "unexpected stderr: {stderr}");
    serde_json::from_str(stdout).unwrap_or_else(|error| panic!("{error}: {stdout}\n{stderr}"))
}

fn expected_library(corpus: &Value, broken: bool, declaration: bool) -> Value {
    let mut files = corpus["expected"]["brokenFiles"].clone();
    if !broken {
        for file in files.as_array_mut().unwrap() {
            file["diagnostics"] = json!([]);
        }
    }
    let mut result = json!({"files":files,"programs":[{"root":".","tsconfig":"tsconfig.json","compilerOptions":corpus["expected"]["compilerOptions"],"files":corpus["expected"]["programFiles"]}],"errorCount":usize::from(broken),"warningCount":0,"fileCount":2});
    if declaration {
        result["declarations"] = corpus["expected"]["declarations"].clone();
    }
    result
}

#[test]
fn diamond_default_project_reports_whole_authored_selection_and_repair() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let corpus = fixture();
    let root = create_library("diamond-default-selection", &corpus);
    assert_eq!(
        check(&root, &corsa, false, 1),
        expected_library(&corpus, true, false)
    );
    std::fs::write(
        root.join("src/base/main.ts"),
        corpus["repair"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(
        check(&root, &corsa, false, 0),
        expected_library(&corpus, false, false)
    );
    // Reversing only config order must select the sibling's clean root. Every
    // request starts a fresh process, so no stale merged spec may survive.
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"extends":["./configs/second.json","./configs/first.json"]}"#,
    )
    .unwrap();
    let mut expected_reversed = expected_library(&corpus, false, false);
    expected_reversed["files"] = json!([{"file":"src/first/first.ts","diagnostics":[]}]);
    expected_reversed["programs"][0]["files"] = json!(["src/first/first.ts"]);
    expected_reversed["fileCount"] = json!(1);
    expected_reversed["programs"][0]["compilerOptions"] =
        corpus["expected"]["reverseCompilerOptions"].clone();
    assert_eq!(check(&root, &corsa, false, 0), expected_reversed);
    std::fs::write(
        root.join("tsconfig.json"),
        corpus["files"]["tsconfig.json"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(
        check(&root, &corsa, false, 0),
        expected_library(&corpus, false, false)
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn diamond_declarations_maps_and_packed_vue_consumer_keep_complete_types() {
    let Some(corsa) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let corpus = fixture();
    let root = create_library("diamond-declaration-library", &corpus);
    std::fs::write(
        root.join("src/base/main.ts"),
        corpus["repair"].as_str().unwrap(),
    )
    .unwrap();
    // No --declaration-dir: the entire inherited destination/map contract must
    // come from the shared ancestor of the later sibling.
    assert_eq!(
        check(&root, &corsa, true, 0),
        expected_library(&corpus, false, true)
    );
    assert!(!root.join("types-first").exists());
    assert!(!root.join("dist-first").exists());
    assert_eq!(
        std::fs::read_to_string(root.join("types-base/main.d.ts")).unwrap(),
        DECLARATION
    );
    let map: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("types-base/main.d.ts.map")).unwrap(),
    )
    .unwrap();
    assert_eq!(map, serde_json::from_str::<Value>(DECLARATION_MAP).unwrap());
    let vue_map: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("types-base/App.vue.d.ts.map")).unwrap(),
    )
    .unwrap();
    assert_eq!(vue_map["sourceRoot"], "");
    assert_eq!(vue_map["sources"], json!(["../src/base/App.vue"]));
    assert_eq!(
        std::fs::read_to_string(
            root.join("types-base")
                .join(vue_map["sources"][0].as_str().unwrap())
        )
        .unwrap(),
        corpus["files"]["src/base/App.vue"].as_str().unwrap()
    );

    let consumer = create_cli_project(
        "diamond-declaration-consumer",
        &[("src/index.ts", corpus["consumer"].as_str().unwrap())],
    );
    vue_stub::install_vue_jsx_type_stub(&consumer);
    let package = consumer.join("node_modules/@scope/diamond");
    copy_emitted_tree(&root.join("types-base"), &package);
    std::fs::write(package.join("package.json"), r#"{"name":"@scope/diamond","types":"./main.d.ts","exports":{".":{"types":"./main.d.ts"},"./App.vue":{"types":"./App.vue.d.ts"}}}"#).unwrap();
    // A deleted library cannot satisfy an accidental source-workspace link.
    std::fs::remove_dir_all(&root).unwrap();
    assert!(!root.exists());
    assert_eq!(
        check(&consumer, &corsa, false, 0),
        expected_consumer(&corpus, false)
    );
    std::fs::write(
        consumer.join("src/index.ts"),
        corpus["consumerBroken"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(
        check(&consumer, &corsa, false, 1),
        expected_consumer(&corpus, true)
    );
    std::fs::write(
        consumer.join("src/index.ts"),
        corpus["consumer"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(
        check(&consumer, &corsa, false, 0),
        expected_consumer(&corpus, false)
    );
    std::fs::remove_dir_all(consumer).unwrap();
}

fn expected_consumer(corpus: &Value, broken: bool) -> Value {
    let files = if broken {
        corpus["expected"]["consumerBrokenFiles"].clone()
    } else {
        json!([{"file":"src/index.ts","diagnostics":[]}])
    };
    json!({"files":files,"programs":[{"root":".","tsconfig":"tsconfig.json","compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"files":["src/index.ts"]}],"errorCount":usize::from(broken),"warningCount":0,"fileCount":1})
}

fn copy_emitted_tree(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        assert!(
            !kind.is_symlink(),
            "packed declarations must be copied files"
        );
        let path = target.join(entry.file_name());
        if kind.is_dir() {
            copy_emitted_tree(&entry.path(), &path);
        } else {
            std::fs::copy(entry.path(), path).unwrap();
        }
    }
}
