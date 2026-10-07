//! Complete original #7988 inputs exercise the real config loader and CLI.

#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "CLI fixtures use std strings")]
#![expect(
    clippy::disallowed_macros,
    reason = "CLI fixture diagnostics use format"
)]

use serde::Deserialize;
use std::{fs, path::Path, process::Command};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    config: String,
    inputs: Vec<String>,
    plain: String,
    json: String,
    exit: i32,
}

fn fixtures() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/props-destructuring-original-7988")
}

fn invoke(root: &Path, format: &str, inputs: &[String]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["lint", "-f", format, "--help-level", "short"])
        .args(inputs)
        .output()
        .expect("execute the source-built CLI")
}

fn preserve_inputs(root: &Path, inputs: &[String], config: &[u8]) {
    assert_eq!(fs::read(root.join("vize.config.json")).unwrap(), config);
    for input in inputs {
        assert_eq!(
            fs::read(root.join(input)).unwrap(),
            fs::read(fixtures().join(format!("{input}.txt"))).unwrap(),
            "original input changed: {input}"
        );
    }
    assert!(!root.join("node_modules").exists());
}

#[test]
fn complete_original_props_styles_match_cli_config_plain_and_json() {
    let cases: Vec<Case> = serde_json::from_slice(
        &fs::read(fixtures().join("cases.json")).expect("committed whole-output corpus"),
    )
    .unwrap();
    assert_eq!(cases.len(), 12);
    let mut comparisons = 0;
    for case in cases {
        let root = tempfile::tempdir().unwrap();
        let config = fs::read(fixtures().join(&case.config)).unwrap();
        fs::write(root.path().join("vize.config.json"), &config).unwrap();
        for input in &case.inputs {
            fs::copy(
                fixtures().join(format!("{input}.txt")),
                root.path().join(input),
            )
            .unwrap();
        }
        let expected_plain = fs::read(fixtures().join(&case.plain)).unwrap();
        let expected_json: serde_json::Value =
            serde_json::from_slice(&fs::read(fixtures().join(&case.json)).unwrap()).unwrap();
        for format in ["plain", "json"] {
            let mut previous = None;
            for _ in 0..2 {
                let output = invoke(root.path(), format, &case.inputs);
                assert_eq!(output.status.code(), Some(case.exit), "{}", case.id);
                assert!(
                    output.stderr.is_empty(),
                    "{}: {}",
                    case.id,
                    String::from_utf8_lossy(&output.stderr)
                );
                if format == "plain" {
                    assert_eq!(
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&expected_plain),
                        "{}: complete plain output",
                        case.id
                    );
                } else {
                    let actual: serde_json::Value =
                        serde_json::from_slice(&output.stdout).expect("complete CLI JSON result");
                    assert_eq!(actual, expected_json, "{}: complete JSON output", case.id);
                }
                if let Some(previous) = &previous {
                    assert_eq!(&output.stdout, previous, "{}: repeated output", case.id);
                }
                previous = Some(output.stdout);
                preserve_inputs(root.path(), &case.inputs, &config);
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 48);
    println!(
        "props original CLI corpus: original_files=2 mode_cases=12 formats=2 repetitions=2 comparisons=48"
    );
}

#[test]
fn invalid_props_modes_fail_closed_before_linting_the_original_inputs() {
    for (options, expected) in [
        (
            serde_json::json!({ "destructure": "sometimes" }),
            "unknown variant `sometimes`, expected one of `only-when-assigned`, `always`, `never` at line 6 column 34",
        ),
        (
            serde_json::json!({ "other": true }),
            "unknown field `other`, expected `destructure` at line 6 column 15",
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        let mut config: serde_json::Value = serde_json::from_slice(
            &fs::read(fixtures().join("original-vize.config.json.txt")).unwrap(),
        )
        .unwrap();
        config["linter"]["ruleOptions"] =
            serde_json::json!({ "script/define-props-destructuring": options });
        let config = serde_json::to_vec_pretty(&config).unwrap();
        fs::write(root.path().join("vize.config.json"), &config).unwrap();
        let inputs = vec!["MyBadgeA.vue".into(), "MyBadgeB.vue".into()];
        for input in &inputs {
            fs::copy(
                fixtures().join(format!("{input}.txt")),
                root.path().join(input),
            )
            .unwrap();
        }
        for _ in 0..2 {
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(root.path())
                .args([
                    "lint",
                    "--config",
                    "vize.config.json",
                    "-f",
                    "json",
                    "--help-level",
                    "short",
                ])
                .args(&inputs)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert_eq!(
                stderr,
                format!("\u{1b}[31mError:\u{1b}[0m failed to parse vize.config.json: {expected}\n")
            );
            preserve_inputs(root.path(), &inputs, &config);
        }
    }
}
