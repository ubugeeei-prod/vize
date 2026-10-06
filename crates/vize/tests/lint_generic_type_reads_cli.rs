use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const ROOT: &str = "../../tests/_fixtures/differential/linter/generic-type-reads-7938/";
const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/linter/generic-type-reads-7938/cases.json");
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/generic-type-reads-7938/vize.config.json.txt"
);
const KIND: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/generic-type-reads-7938/kind.ts.txt"
);
const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/generic-type-reads-7938/kind-picker.vue.txt"
);
const COMMAND: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/generic-type-reads-7938/original-command.txt"
);

fn invoke(root: &Path, args: &[&str], id: &str) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    // Whole bytes and status are retained before assertions. Text mode includes
    // an elapsed-time summary; it is evidence, not a guessed timing oracle.
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

fn unchanged(root: &Path, source: &str) {
    assert_eq!(
        fs::read(root.join("src/kind-picker.vue")).unwrap(),
        source.as_bytes()
    );
    assert_eq!(fs::read(root.join("src/kind.ts")).unwrap(), KIND.as_bytes());
    assert_eq!(
        fs::read(root.join("vize.config.json")).unwrap(),
        CONFIG.as_bytes()
    );
}

#[test]
fn original_exact_argv_and_all_whole_configured_json_vectors_are_checked() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("vize.config.json"), CONFIG).unwrap();
    fs::write(root.join("src/kind.ts"), KIND).unwrap();
    fs::write(root.join("src/kind-picker.vue"), ORIGINAL).unwrap();
    let original_args: Vec<_> = COMMAND.split_whitespace().skip(1).collect();
    assert_eq!(
        original_args,
        ["lint", "-c", "vize.config.json", "src/kind-picker.vue"]
    );
    invoke(root, &original_args, "whole-original-command");
    unchanged(root, ORIGINAL);
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 19);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        fs::write(root.join("src/kind-picker.vue"), source).unwrap();
        let output = invoke(
            root,
            &[
                "lint",
                "-c",
                "vize.config.json",
                "--format",
                "json",
                "src/kind-picker.vue",
            ],
            case["id"].as_str().unwrap(),
        );
        let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(actual, case["cli"], "{}: {output:?}", case["id"]);
        unchanged(root, source);
    }
}

#[test]
fn unused_rule_remains_opt_in_and_original_fixture_bytes_are_literal() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("vize.config.json"), CONFIG).unwrap();
    fs::write(root.join("src/kind.ts"), KIND).unwrap();
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let source = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "no-generic-keeps-original-warning")
        .unwrap()["source"]
        .as_str()
        .unwrap();
    fs::write(root.join("src/kind-picker.vue"), source).unwrap();
    let output = invoke(
        root,
        &[
            "lint",
            "--no-config",
            "--format",
            "json",
            "src/kind-picker.vue",
        ],
        "default-preset",
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!([{
            "file": "src/kind-picker.vue", "messages": [], "errorCount": 0, "warningCount": 0
        }])
    );
    unchanged(root, source);
    let inputs = Path::new(env!("CARGO_MANIFEST_DIR")).join(ROOT);
    for (name, bytes) in [
        ("kind-picker.vue.txt", ORIGINAL),
        ("kind.ts.txt", KIND),
        ("vize.config.json.txt", CONFIG),
        ("original-command.txt", COMMAND),
    ] {
        assert_eq!(fs::read(inputs.join(name)).unwrap(), bytes.as_bytes());
    }
}
