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
    path::{Path, PathBuf},
    process::Command,
};
use vize_canon::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};

mod element_plus;
mod handoff_notice;
mod transport_refusal;
use transport_refusal::without_transport;

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

fn retain_packet(name: &str, packet: &serde_json::Value) {
    if let Some(destination) = std::env::var_os("VIZE_NATIVE_SYNTAX_CAPTURE_DIR") {
        let directory = PathBuf::from(destination);
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join(format!("{name}.json")),
            serde_json::to_vec_pretty(packet).unwrap(),
        )
        .unwrap();
    }
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
    let original_stdout = String::from_utf8(output.stdout).unwrap();
    let original_stderr = String::from_utf8(output.stderr).unwrap();
    retain_packet(
        "literal-cli-before",
        &serde_json::json!({
            "native":native, "cwd":before.path(),
            "argv":["--checkers","1","--pretty","false","--project",before.path().join("tsconfig.json").to_str().unwrap()],
            "wholeConfig":CONFIG,
            "wholeInjectedGeneratedSource":fs::read_to_string(before.path().join("Injected.vue.ts")).unwrap(),
            "wholeAuthoredDeclaration":fs::read_to_string(before.path().join("post-request.d.ts")).unwrap(),
            "exitCode":output.status.code(), "wholeStdout":original_stdout, "wholeStderr":original_stderr,
        }),
    );
    assert!(!output.status.success());
    assert_eq!(
        original_stdout,
        "post-request.d.ts(1294,1): error TS1128: Declaration or statement expected.\n"
    );
    assert_eq!(original_stderr, "");

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
    let virtual_root = vize_canon::project_virtual_root(project.path());
    let direct = Command::new(&native)
        .args(["--checkers", "1", "--pretty", "false", "--project"])
        .arg(virtual_root.join("tsconfig.json"))
        .current_dir(&virtual_root)
        .output()
        .unwrap();
    let stdout = String::from_utf8(direct.stdout).unwrap();
    let stderr = String::from_utf8(direct.stderr).unwrap();
    let attempts = fs::read_to_string(project.path().join(".bin/denied-transports.log")).unwrap();
    let denied_bootstraps = transport_refusal::capture_and_replay(project.path(), &wrapper);
    retain_packet(
        "unavailable-transport",
        &serde_json::json!({
            "native":native, "wrapper":wrapper,
            "wholeWrapper":fs::read_to_string(&wrapper).unwrap(),
            "wholeDeniedTransportTrace":attempts,
            "wholeDeniedBootstrapPackets":denied_bootstraps,
            "cwd":virtual_root, "wholeGeneratedConfig":fs::read_to_string(virtual_root.join("tsconfig.json")).unwrap(),
            "exitCode":direct.status.code(), "wholeStdout":stdout, "wholeStderr":stderr,
            "wholeError":format!("{error:#?}"),
        }),
    );
    assert_eq!(direct.status.code(), Some(1));
    assert_eq!(
        stdout,
        "post-request.d.cts(1294,1): error TS1128: Declaration or statement expected.\n"
    );
    assert_eq!(stderr, "");
    assert!(
        !attempts.is_empty(),
        "semantic completion must try a transport"
    );
    assert!(
        attempts
            .split_inclusive('\n')
            .all(|record| matches!(record, "--api\n" | "--lsp\n")),
        "only actual denied diagnostic transports may enter the trace: {attempts:?}"
    );
    let cli_error = format!(
        "corsa error (exit code {}): Corsa CLI semantic diagnostics are incomplete after a syntax error:\n{}",
        direct.status.code().unwrap(),
        stdout.trim()
    );
    let vize_canon::CorsaError::CorsaExecution { exit_code, message } = error else {
        panic!("native completion must return an execution failure: {error:#?}");
    };
    assert_eq!(exit_code, 1);
    let api_completion = format!(
        "{cli_error}\nSame-config native diagnostic completion failed: corsa error (exit code -1): Failed to start Corsa API session: rpc error -32041: native diagnostic transport intentionally unavailable"
    );
    let message = message.as_str();
    let project_suffix = message.strip_prefix(api_completion.as_str()).unwrap_or("");
    let names_project = project_suffix.len() >= 10 && &project_suffix[..10] == " (project ";
    assert!(message == api_completion || names_project, "{message}");
    transport_refusal::assert_exact_denial(&denied_bootstraps, &attempts);
    println!("{message}");
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
    let attempts = fs::read_to_string(project.path().join(".bin/denied-transports.log")).unwrap();
    retain_packet(
        "ordinary-semantic-fast-path",
        &serde_json::json!({
            "native":native, "wrapper":wrapper,
            "wholeWrapper":fs::read_to_string(&wrapper).unwrap(),
            "wholeDeniedTransportTrace":attempts,
            "wholeResult":format!("{result:#?}"),
        }),
    );
    assert_eq!(attempts, "", "ordinary CLI semantics must not try API/LSP");
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
