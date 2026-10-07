//! Whole public CLI vectors for #7935, with the complete #7932 Nuxt inputs.
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/cross-file-rule-severity")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn write(root: &Path, path: &str, bytes: &[u8]) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, bytes).unwrap();
}

fn inputs(root: &Path, paths: &[String]) -> Value {
    Value::Array(
        paths
            .iter()
            .map(|path| {
                let bytes = fs::read(root.join(path)).unwrap();
                json!({"path": path, "bytes": bytes.len(), "sha256": hash(&bytes)})
            })
            .collect(),
    )
}

fn capture_root() -> PathBuf {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".into());
    let parent = workspace
        .join("target/nextest")
        .join(profile)
        .join("cross-file-rule-severity-7935");
    fs::create_dir_all(&parent).unwrap();
    // Concurrent invocations retain independent complete process evidence.
    tempfile::Builder::new()
        .prefix("invocation-")
        .tempdir_in(parent)
        .unwrap()
        .keep()
}

fn custody(root: &Path) {
    let catalog = read_json(&root.join("catalog.json"));
    for file in catalog.as_array().unwrap() {
        let bytes = fs::read(root.join(file["path"].as_str().unwrap())).unwrap();
        assert_eq!(json!(bytes.len()), file["bytes"]);
        assert_eq!(json!(hash(&bytes)), file["sha256"]);
    }
    let report = fs::read_to_string(root.join("issue-7932.md")).unwrap();
    for (language, path) in [
        (
            "vue",
            "inputs/reported-nuxt/src/components/user-link.vue.txt",
        ),
        (
            "ts",
            "inputs/reported-nuxt/src/components/user-link.test.ts.txt",
        ),
    ] {
        let source = fs::read_to_string(root.join(path)).unwrap();
        assert_eq!(fenced_blocks(&report, language), [source]);
    }
    let report = fs::read_to_string(root.join("issue-7935.md")).unwrap();
    for (language, path) in [
        ("json", "original-vize.config.json.txt"),
        ("sh", "original-command.txt"),
    ] {
        let source = fs::read_to_string(root.join(path)).unwrap();
        assert_eq!(fenced_blocks(&report, language), [source]);
    }
}

fn fenced_blocks(report: &str, language: &str) -> Vec<String> {
    report
        .split(&format!("```{language}\n"))
        .skip(1)
        .map(|block| block.split("```").next().unwrap().to_owned())
        .collect()
}

#[test]
fn configured_cross_file_findings_keep_complete_reports_and_input_bytes() {
    let corpus = corpus();
    custody(&corpus);
    let manifest = read_json(&corpus.join("cases.json"));
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 74);
    let capture = capture_root();
    let catalog_bytes = fs::read(corpus.join("catalog.json")).unwrap();
    fs::write(
        capture.join("authority.json"),
        serde_json::to_vec_pretty(&json!({
            "sourceSha": std::env::var("SOURCE_SHA").ok(),
            "cliBinary": env!("CARGO_BIN_EXE_vize"),
            "cliSha256": hash(&fs::read(env!("CARGO_BIN_EXE_vize")).unwrap()),
            "catalogSha256": hash(&catalog_bytes),
            "casesSha256": hash(&fs::read(corpus.join("cases.json")).unwrap()),
            "expectedCases": 74, "expectedInvocations": 148
        }))
        .unwrap(),
    )
    .unwrap();
    let mut completed = Vec::new();
    let mut file_rows = 0;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let project = case["project"].as_str().unwrap();
        let supplied = read_json(&corpus.join(format!("inputs/{project}.json")));
        let expected = read_json(&corpus.join(format!("cases/{name}.json")));
        assert_eq!(json!(expected.as_array().unwrap().len()), case["files"]);
        let errors: u64 = expected
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["errorCount"].as_u64().unwrap())
            .sum();
        let warnings: u64 = expected
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["warningCount"].as_u64().unwrap())
            .sum();
        assert_eq!(json!(errors), case["errors"]);
        assert_eq!(json!(warnings), case["warnings"]);
        for format in ["json", "plain"] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path();
            let mut paths = Vec::new();
            for input in supplied.as_array().unwrap() {
                let path = input["path"].as_str().unwrap();
                let bytes = fs::read(corpus.join(input["source"].as_str().unwrap())).unwrap();
                write(root, path, &bytes);
                paths.push(path.to_owned());
            }
            let config = fs::read(corpus.join(case["config"].as_str().unwrap())).unwrap();
            write(root, "vize.config.json", &config);
            paths.push("vize.config.json".into());
            paths.sort();
            let before = inputs(root, &paths);
            let original = fs::read_to_string(corpus.join("original-command.txt")).unwrap();
            let mut args: Vec<String> = original
                .split_whitespace()
                .skip(1)
                .map(|arg| arg.trim_matches('"').to_owned())
                .collect();
            assert_eq!(
                args,
                [
                    "lint",
                    "-c",
                    "vize.config.json",
                    "--cross-file",
                    "src/**/*.vue",
                    "src/**/*.ts"
                ]
            );
            args.extend([
                "--format".into(),
                format.into(),
                "--help-level".into(),
                case["helpLevel"].as_str().unwrap().into(),
            ]);
            args.extend(
                case["extraArgs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|arg| arg.as_str().unwrap().to_owned()),
            );
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(root)
                .args(&args)
                .output()
                .unwrap();
            let evidence = capture.join(name).join(format);
            fs::create_dir_all(&evidence).unwrap();
            fs::write(evidence.join("stdout.txt"), &output.stdout).unwrap();
            fs::write(evidence.join("stderr.txt"), &output.stderr).unwrap();
            let after = inputs(root, &paths);
            fs::write(
                evidence.join("process.json"),
                serde_json::to_vec_pretty(&json!({
                    "case": name, "binary": env!("CARGO_BIN_EXE_vize"),
                    "argv": args, "status": output.status.code(),
                    "before": before, "after": after
                }))
                .unwrap(),
            )
            .unwrap();
            assert_eq!(
                output.status.code(),
                Some(case["status"].as_i64().unwrap() as i32),
                "{name}/{format}: {output:?}"
            );
            assert_eq!(
                output.stderr,
                fs::read(corpus.join(format!("cases/{name}.stderr.txt"))).unwrap(),
                "{name}/{format}"
            );
            assert_eq!(
                output.stdout,
                fs::read(corpus.join(format!("cases/{name}.{format}.stdout.txt"))).unwrap(),
                "{name}/{format}: {output:?}"
            );
            if format == "json" {
                assert_eq!(
                    serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                    expected,
                    "{name}"
                );
                file_rows += expected.as_array().unwrap().len();
            }
            for input in supplied.as_array().unwrap() {
                assert_eq!(
                    fs::read(root.join(input["path"].as_str().unwrap())).unwrap(),
                    fs::read(corpus.join(input["source"].as_str().unwrap())).unwrap(),
                    "{name}: unchanged input"
                );
            }
            assert_eq!(
                fs::read(root.join("vize.config.json")).unwrap(),
                config,
                "{name}: unchanged config"
            );
            assert_eq!(before, after, "{name}/{format}: whole input custody");
            completed.push(format!("{name}/{format}"));
        }
    }
    assert_eq!(completed.len(), 148);
    assert_eq!(file_rows, 252);
    fs::write(
        capture.join("completed.json"),
        serde_json::to_vec_pretty(&json!({"completed": completed, "jsonFileRows": file_rows}))
            .unwrap(),
    )
    .unwrap();
}
