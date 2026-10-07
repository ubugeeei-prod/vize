//! #7928: existing formatted project JSON must pass `fmt --check` unchanged.
#![expect(
    clippy::disallowed_macros,
    reason = "CLI fixtures use standard strings"
)]
#![expect(clippy::disallowed_types, reason = "CLI fixtures use standard strings")]

use serde::Deserialize;
use std::{fs, path::Path, process::Command};

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    file: String,
    source: String,
    expected: String,
    #[serde(default)]
    options: serde_json::Value,
}

fn fmt(root: &Path, case: &Case, mode: &str) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command.current_dir(root).args(["fmt", mode]);
    if case.options.is_null() {
        command.arg("--no-config");
    } else {
        command.args(["--config", "vize.config.json"]);
    }
    command.arg(&case.file).output().unwrap()
}

#[test]
fn json_layout_corpus_has_correct_check_write_and_second_check_behavior() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-regressions/json-layout-7928/cases.json"
    ))
    .unwrap();
    for case in corpus.cases {
        let project = tempfile::tempdir().unwrap();
        let file = project.path().join(&case.file);
        fs::write(&file, &case.source).unwrap();
        let config = serde_json::json!({ "formatter": case.options });
        fs::write(
            project.path().join("vize.config.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();

        let check = fmt(project.path(), &case, "--check");
        assert_eq!(
            check.status.code(),
            Some(i32::from(case.source != case.expected)),
            "{}: initial check: {}",
            case.id,
            String::from_utf8_lossy(&check.stderr)
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), case.source);
        let write = fmt(project.path(), &case, "--write");
        assert!(
            write.status.success(),
            "{}: write: {}",
            case.id,
            String::from_utf8_lossy(&write.stderr)
        );
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            case.expected,
            "{}",
            case.id
        );
        let check = fmt(project.path(), &case, "--check");
        assert!(
            check.status.success(),
            "{}: second check: {}",
            case.id,
            String::from_utf8_lossy(&check.stderr)
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), case.expected);
    }
}
