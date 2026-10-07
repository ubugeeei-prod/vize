use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/linter/call-source-7989/cases.json");
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/call-source-7989/vize.config.json.txt"
);
const ORIGINAL: &str =
    include_str!("../../../tests/_fixtures/differential/linter/call-source-7989/MyNotes.vue.txt");
const COMMAND: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/call-source-7989/original-command.txt"
);
const PARTIAL_PLAIN: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/call-source-7989/partial-original-plain.txt"
);

fn invoke(root: &Path, args: &[&str], id: &str) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    eprintln!(
        "{}",
        json!({"case": id, "binary": env!("CARGO_BIN_EXE_vize"),
        "argv": args, "status": output.status.code(), "stdout": output.stdout,
        "stderr": output.stderr})
    );
    assert_eq!(output.status.code(), Some(0), "{id}: {output:?}");
    assert_eq!(output.stderr, Vec::<u8>::new(), "{id}: {output:?}");
    output
}

fn unchanged(root: &Path, filename: &str, source: &str) {
    assert_eq!(fs::read(root.join(filename)).unwrap(), source.as_bytes());
    assert_eq!(
        fs::read(root.join("vize.config.json")).unwrap(),
        CONFIG.as_bytes()
    );
}

#[test]
fn original_argv_and_complete_comment_string_real_call_vectors_are_checked() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::write(root.join("vize.config.json"), CONFIG).unwrap();
    fs::write(root.join("MyNotes.vue"), ORIGINAL).unwrap();
    let original_args: Vec<_> = COMMAND.split_whitespace().skip(1).collect();
    assert_eq!(
        original_args,
        ["lint", "-f", "plain", "--help-level", "none", "MyNotes.vue"]
    );
    let output = invoke(root, &original_args, "whole-original-command-partial");
    assert_eq!(output.stdout, PARTIAL_PLAIN.as_bytes(), "{output:?}");
    unchanged(root, "MyNotes.vue", ORIGINAL);
    fs::remove_file(root.join("MyNotes.vue")).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 41);
    assert_eq!(cases[0]["source"].as_str().unwrap(), ORIGINAL);
    for case in cases {
        let filename = case["filename"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        fs::write(root.join(filename), source).unwrap();
        let output = invoke(
            root,
            &["lint", "--format", "json", filename],
            case["id"].as_str().unwrap(),
        );
        let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(actual, case["cli"], "{}: {output:?}", case["id"]);
        unchanged(root, filename, source);
        fs::remove_file(root.join(filename)).unwrap();
    }
}
