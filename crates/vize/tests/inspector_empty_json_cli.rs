use std::process::Command;

#[test]
fn inspector_json_emits_an_empty_payload_when_patterns_match_no_files() {
    let project = tempfile::tempdir().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .args(["inspector", "**/*.vue", "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stderr, "No .vue files found matching the patterns\n");

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["selectedFile"], serde_json::Value::Null);
    assert_eq!(json["files"], serde_json::json!([]));
}
