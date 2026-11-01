//! Public build commands consume the same original inputs as the SFC API.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "whole original-source CLI process and output regression evidence"
)]

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[derive(Deserialize)]
struct Case {
    id: String,
    file: String,
    filename: String,
    expect: String,
    bytes: usize,
    sha256: String,
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn cases() -> Vec<(Case, String)> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler/sfc-entry-validation-11262-11263");
    let raw = fs::read(directory.join("cases.json")).unwrap();
    assert_eq!(
        sha256(&raw),
        "a16ca0e07eaefff3925e7f6a015b6e22396034c4915a6ed518ecc7cc7f9e4710"
    );
    let json: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    let cases: Vec<Case> = serde_json::from_value(json["cases"].clone()).unwrap();
    assert_eq!(cases.len(), 20);
    cases
        .into_iter()
        .map(|case| {
            let source = fs::read(directory.join(&case.file)).unwrap();
            assert_eq!(source.len(), case.bytes);
            assert_eq!(sha256(&source), case.sha256);
            (case, String::from_utf8(source).unwrap())
        })
        .collect()
}

fn run(root: &Path, name: &str, format: &str, vapor: bool) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .env("RAYON_NUM_THREADS", "1")
        .args([
            "build",
            "--no-config",
            "--slow-threshold",
            "600000",
            "-o",
            "out",
            "-f",
            format,
            name,
        ]);
    if vapor {
        command.arg("--vapor");
    }
    command.output().unwrap()
}

fn normalize_time(bytes: &[u8]) -> String {
    let text = std::str::from_utf8(bytes).unwrap();
    let (head, time) = text.rsplit_once(" compiled in ").unwrap();
    let (time, tail) = time.split_once('s').unwrap();
    let (seconds, decimal) = time.split_once('.').unwrap();
    assert!(!seconds.is_empty() && seconds.bytes().all(|byte| byte.is_ascii_digit()));
    assert_eq!(decimal.len(), 4);
    assert!(decimal.bytes().all(|byte| byte.is_ascii_digit()));
    assert!(time.parse::<f64>().unwrap().is_finite());
    format!("{head} compiled in <TIME>s{tail}")
}

fn retain(
    case: &Case,
    vapor: bool,
    format: &str,
    output: &std::process::Output,
    artifact: Option<&[u8]>,
) {
    println!(
        "sfc-entry-cli {}",
        serde_json::json!({
            "id": case.id, "sourceSha256": case.sha256, "vapor": vapor, "format": format,
            "status": output.status.code(), "stdout": output.stdout, "stderr": output.stderr,
            "artifact": artifact,
        })
    );
}

#[test]
fn invalid_sfc_entries_fail_without_emitting_a_module() {
    for (case, source) in cases()
        .into_iter()
        .filter(|(case, _)| case.expect != "valid")
    {
        let message = match case.expect.as_str() {
            "export-error" => "<script setup> cannot contain ES module exports.",
            "missing-block" => {
                "At least one <template> or <script> is required in a single file component."
            }
            other => panic!("unknown expectation {other}"),
        };
        for vapor in [false, true] {
            for format in ["js", "json"] {
                let project = tempfile::tempdir().unwrap();
                fs::write(project.path().join(&case.filename), &source).unwrap();
                let output = run(project.path(), &case.filename, format, vapor);
                retain(&case, vapor, format, &output, None);
                assert_eq!(output.status.code(), Some(1), "{}", case.id);
                assert_eq!(output.stdout, b"");
                assert_eq!(
                    normalize_time(&output.stderr),
                    format!(
                        "\n\x1b[31m✗ 1 error(s) occurred:\x1b[0m\n\n  \x1b[31mCompile errors (1):\x1b[0m\n    \x1b[1m{}\x1b[0m\n      {message}\n\n\x1b[31m✗ 1 file(s) failed\x1b[0m, 0 compiled in <TIME>s\n",
                        case.filename
                    )
                );
                assert!(!project.path().join("out").exists());
                assert_eq!(
                    fs::read_to_string(project.path().join(&case.filename)).unwrap(),
                    source
                );
            }
        }
    }
}

#[test]
fn valid_type_only_and_single_block_controls_keep_complete_build_outputs() {
    for (case, source) in cases()
        .into_iter()
        .filter(|(case, _)| case.expect == "valid")
    {
        for vapor in [false, true] {
            let descriptor = parse_sfc(
                &source,
                SfcParseOptions {
                    filename: case.filename.as_str().into(),
                    ..Default::default()
                },
            )
            .unwrap();
            let mut options = SfcCompileOptions::default();
            options.parse.filename = case.filename.as_str().into();
            options.script.id = Some(case.filename.as_str().into());
            options.template.id = Some(case.filename.as_str().into());
            options.style.id = case.filename.as_str().into();
            options.vapor = vapor;
            let script_lang = descriptor
                .script_setup
                .as_ref()
                .or(descriptor.script.as_ref())
                .and_then(|script| script.lang.as_deref())
                .unwrap_or("js");
            let compiled = compile_sfc(&descriptor, options).unwrap();
            assert!(compiled.errors.is_empty() && compiled.warnings.is_empty());
            for format in ["js", "json"] {
                let project = tempfile::tempdir().unwrap();
                fs::write(project.path().join(&case.filename), &source).unwrap();
                let output = run(project.path(), &case.filename, format, vapor);
                let artifact = project
                    .path()
                    .join("out")
                    .join(case.filename.replace(".vue", &format!(".{format}")));
                let bytes = fs::read(&artifact).ok();
                retain(&case, vapor, format, &output, bytes.as_deref());
                assert_eq!(output.status.code(), Some(0), "{}", case.id);
                assert_eq!(output.stdout, b"");
                assert_eq!(
                    normalize_time(&output.stderr),
                    format!(
                        "Built: {} -> out/{}\n\x1b[32m✓ 1 file compiled in <TIME>s\x1b[0m\n",
                        case.filename,
                        case.filename.replace(".vue", &format!(".{format}"))
                    )
                );
                let bytes = bytes.unwrap();
                if format == "js" {
                    assert_eq!(bytes, compiled.code.as_bytes());
                } else {
                    let whole: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                    assert_eq!(
                        whole,
                        serde_json::json!({
                            "filename": case.filename, "code": compiled.code, "css": compiled.css,
                            "errors": [], "warnings": [],
                        "script_lang": script_lang,
                            "macro_artifacts": [],
                        })
                    );
                }
                assert_eq!(
                    fs::read_to_string(project.path().join(&case.filename)).unwrap(),
                    source
                );
            }
        }
    }
}
