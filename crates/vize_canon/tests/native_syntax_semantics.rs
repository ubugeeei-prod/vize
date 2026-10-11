//! Actual native parser aborts must not hide unrelated semantic diagnostics.
#![cfg(unix)]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "whole native fixture packets use std strings"
)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};
use vize_canon::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/native-syntax-semantic-completion/"
);
const CONFIG: &str = r#"{"compilerOptions":{"strict":true,"noEmit":true,"skipLibCheck":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","types":[]},"include":["*.ts","*.vue"]}"#;

fn native() -> Option<PathBuf> {
    let required = std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref() == Ok("1");
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        assert!(
            !required,
            "required native syntax/semantic laws cannot be disabled"
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
        .expect("actual native runtime is required")
        .canonicalize()
        .unwrap(),
    )
}

fn install(root: &Path, broken: bool) {
    fs::write(root.join("tsconfig.json"), CONFIG).unwrap();
    fs::write(
        root.join("post-request.d.ts"),
        fs::read(format!("{CORPUS}post-request.d.ts.txt")).unwrap(),
    )
    .unwrap();
    let original = fs::read_to_string(format!("{CORPUS}original-broken.vue.txt")).unwrap();
    let source = if broken {
        original
    } else {
        original.replace("\"not-a-number\"", "1")
    };
    fs::write(root.join("Injected.vue"), source).unwrap();
    // Use the genuine locked Vue package; no replacement declarations or
    // injected diagnostic answers stand in for its public component types.
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    fs::create_dir_all(root.join("node_modules")).unwrap();
    std::os::unix::fs::symlink(
        repo.join("npm/builder/vite/node_modules/vue")
            .canonicalize()
            .unwrap(),
        root.join("node_modules/vue"),
    )
    .unwrap();
}

fn check(
    root: &Path,
    native: &Path,
) -> Result<vize_canon::BatchTypeCheckResult, vize_canon::CorsaError> {
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        root,
        BatchTypeCheckerOptions::default(),
        Some(native),
    )
    .unwrap();
    checker.set_server_count(Some(1));
    checker.set_checker_count(Some(1));
    checker.scan_project().unwrap();
    let result = checker.check_project();
    if let Some(destination) = std::env::var_os("VIZE_NATIVE_SYNTAX_CAPTURE_DIR") {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let virtual_root = vize_canon::project_virtual_root(root);
        let packet = serde_json::json!({
            "native": native,
            "authoredConfig": fs::read_to_string(root.join("tsconfig.json")).unwrap(),
            "generatedConfig": fs::read_to_string(virtual_root.join("tsconfig.json")).unwrap(),
            "generatedFiles": checker.virtual_files().into_iter().map(|file| serde_json::json!({
                "originalPath":file.original_path, "virtualPath":file.virtual_path,
                "wholeCode":file.content.as_str(), "wholeSourceMap":format!("{:?}",file.source_map),
            })).collect::<Vec<_>>(),
            "wholeResult": format!("{result:#?}"),
        });
        let directory = PathBuf::from(destination);
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join(format!("{sequence:03}.json")),
            serde_json::to_vec_pretty(&packet).unwrap(),
        )
        .unwrap();
    }
    result
}

fn rows(
    root: &Path,
    result: &vize_canon::BatchTypeCheckResult,
) -> Vec<(String, u32, u32, Option<u32>, u8, String)> {
    let root = root.canonicalize().unwrap();
    let mut rows: Vec<_> = result
        .diagnostics
        .iter()
        .map(|d| {
            (
                d.file
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                d.line,
                d.column,
                d.code,
                d.severity,
                d.message.to_string(),
            )
        })
        .collect();
    rows.sort();
    rows
}

fn without_transport(root: &Path, native: &Path) -> PathBuf {
    let wrapper = root.join("native-without-diagnostic-transport.sh");
    let quoted = native.to_string_lossy().replace('\'', "'\"'\"'");
    fs::write(&wrapper, format!("#!/bin/sh\ncase \"$1\" in --api|--lsp) echo 'native diagnostic transport intentionally unavailable' >&2; exit 41;; esac\nexec '{quoted}' \"$@\"\n")).unwrap();
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
    wrapper
}

#[test]
fn actual_native_syntax_and_semantic_errors_survive_and_repair_clears_only_the_semantic_error() {
    let Some(native) = native() else { return };
    let before = tempfile::tempdir().unwrap();
    fs::write(before.path().join("tsconfig.json"), CONFIG).unwrap();
    fs::write(
        before.path().join("post-request.d.ts"),
        fs::read(format!("{CORPUS}post-request.d.ts.txt")).unwrap(),
    )
    .unwrap();
    fs::write(
        before.path().join("Injected.vue.ts"),
        fs::read(format!("{CORPUS}original-broken.vue.ts.txt")).unwrap(),
    )
    .unwrap();
    let output = Command::new(&native)
        .args(["--checkers", "1", "--pretty", "false", "--project"])
        .arg(before.path().join("tsconfig.json"))
        .current_dir(before.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let original_stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        original_stdout.contains(
            "post-request.d.ts(1294,1): error TS1128: Declaration or statement expected."
        ),
        "{original_stdout}"
    );
    assert!(
        !original_stdout.contains("TS2322"),
        "literal native CLI abort control: {original_stdout}"
    );
    assert!(output.stderr.is_empty());

    let syntax = (
        "post-request.d.ts".to_string(),
        1293,
        0,
        Some(1128),
        1,
        "Declaration or statement expected.".to_string(),
    );
    for broken in [true, false] {
        let project = tempfile::tempdir().unwrap();
        install(project.path(), broken);
        let result =
            check(project.path(), &native).expect("same-config native semantics must complete");
        assert!(!result.success);
        let mut expected = vec![syntax.clone()];
        if broken {
            expected.push((
                "Injected.vue".to_string(),
                1,
                6,
                Some(2322),
                1,
                "Type 'string' is not assignable to type 'number'.".to_string(),
            ));
        }
        expected.sort();
        assert_eq!(rows(project.path(), &result), expected);
        // Retain the complete raw CLI control and complete mapped vectors in
        // the hosted native test stream, including the repair's syntax error.
        println!(
            "{}",
            serde_json::json!({"native":native,"originalCliStdout":original_stdout,"broken":broken,"diagnostics":rows(project.path(), &result)})
        );
    }
}

#[test]
fn unavailable_native_semantic_transport_cannot_report_empty_clean() {
    let Some(native) = native() else { return };
    let project = tempfile::tempdir().unwrap();
    install(project.path(), true);
    let wrapper = without_transport(project.path(), &native);
    let error = check(project.path(), &wrapper)
        .expect_err("incomplete native semantics must fail explicitly");
    let whole = error.to_string();
    assert!(
        whole.contains("semantic diagnostics are incomplete after a syntax error"),
        "{whole}"
    );
    assert!(
        whole.contains("TS1128: Declaration or statement expected."),
        "original native body must survive transport failure: {whole}"
    );
    println!("{whole}");
}

#[test]
fn ordinary_semantic_errors_keep_the_cli_fast_path_without_a_diagnostic_transport() {
    let Some(native) = native() else { return };
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    fs::write(
        project.path().join("source.ts"),
        "export const value: number = \"wrong\";\n",
    )
    .unwrap();
    let wrapper = without_transport(project.path(), &native);
    let result =
        check(project.path(), &wrapper).expect("complete native CLI semantics need no API/LSP");
    assert!(!result.success);
    assert_eq!(
        rows(project.path(), &result),
        vec![(
            "source.ts".to_string(),
            0,
            13,
            Some(2322),
            1,
            "Type 'string' is not assignable to type 'number'.".to_string()
        )]
    );
}
