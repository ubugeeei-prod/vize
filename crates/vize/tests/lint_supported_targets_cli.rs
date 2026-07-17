use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-lint-targets-cli-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

fn output_details(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn lint_reports_all_supported_extensions_when_patterns_match_no_files() {
    let project_root = temp_project_dir("no-matching-files");
    fs::create_dir_all(&project_root).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--no-config", "src/**/*.nothing"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_details(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "No .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx files found"
        ),
        "{stderr}"
    );

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn lint_json_emits_an_empty_array_when_patterns_match_no_files() {
    let project_root = temp_project_dir("json-no-matching-files");
    fs::create_dir_all(&project_root).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--no-config", "**/*.vue", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", output_details(&output));
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(stdout, serde_json::json!([]));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "No .vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, or .tsx files found"
        ),
        "{stderr}"
    );

    let _ = fs::remove_dir_all(project_root);
}
