//! Bidirectional semicolon overrides retain every other configured preference.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "whole CLI fixture outputs")]

use serde::Deserialize;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    file: String,
    parser: String,
    source: String,
    with_semicolons: String,
    without_semicolons: String,
}

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

fn corpus() -> Corpus {
    serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/semicolon-cli-6098/cases.json"
    ))
    .unwrap()
}

fn run(root: &Path, file: &str, flags: &[&str], action: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["fmt", "--config", "vite.config.mjs", action])
        .args(flags)
        .arg(file)
        .output()
        .unwrap()
}

#[test]
fn semicolon_overrides_are_bidirectional_and_other_config_bytes_survive() {
    let cases = corpus().cases;
    assert_eq!(cases.len(), 2);
    for case in cases {
        assert!(matches!(case.parser.as_str(), "babel" | "typescript"));
        for (configured, flags, want) in [
            (false, vec![], false),
            (false, vec!["--semi"], true),
            (false, vec!["--semi=true"], true),
            (true, vec![], true),
            (true, vec!["--semi=false"], false),
            (true, vec!["--no-semi"], false),
        ] {
            let project = tempfile::tempdir().unwrap();
            crate::support::vite_config::write(
                project.path(),
                serde_json::to_vec(&serde_json::json!({
                    "formatter": { "semi": configured, "singleQuote": true, "printWidth": 73 }
                }))
                .unwrap(),
            );
            let path = project.path().join(&case.file);
            fs::write(&path, &case.source).unwrap();
            let config = fs::read(project.path().join("vite.config.mjs")).unwrap();
            let expected = if want {
                &case.with_semicolons
            } else {
                &case.without_semicolons
            };
            let before = run(project.path(), &case.file, &flags, "--check");
            assert_eq!(before.status.code(), Some(1), "{before:?}");
            assert_eq!(fs::read_to_string(&path).unwrap(), case.source);
            for _ in 0..2 {
                let written = run(project.path(), &case.file, &flags, "--write");
                assert_eq!(written.status.code(), Some(0), "{written:?}");
                assert_eq!(fs::read_to_string(&path).unwrap(), *expected);
                assert_eq!(
                    fs::read(project.path().join("vite.config.mjs")).unwrap(),
                    config
                );
            }
            let checked = run(project.path(), &case.file, &flags, "--check");
            assert_eq!(checked.status.code(), Some(0), "{checked:?}");
            assert_eq!(fs::read_to_string(&path).unwrap(), *expected);
        }
    }
}

#[test]
fn invalid_semicolon_values_and_conflicting_flags_refuse_before_writes() {
    let project = tempfile::tempdir().unwrap();
    let case = corpus().cases.remove(0);
    crate::support::vite_config::write(project.path(), br#"{"formatter":{"semi":false}}"#);
    let path = project.path().join(&case.file);
    fs::write(&path, &case.source).unwrap();
    let config = fs::read(project.path().join("vite.config.mjs")).unwrap();
    for flags in [
        vec!["--semi=invalid"],
        vec!["--semi", "--no-semi"],
        vec!["--no-semi", "--semi=false"],
    ] {
        let output = run(project.path(), &case.file, &flags, "--write");
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("--semi"), "{stderr}");
        assert!(
            stderr.contains("invalid value") || stderr.contains("cannot be used with"),
            "{stderr}"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), case.source);
        assert_eq!(
            fs::read(project.path().join("vite.config.mjs")).unwrap(),
            config
        );
    }
}
