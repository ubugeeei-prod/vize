use super::{Command, create_cli_project};

#[test]
fn check_json_reports_empty_result_when_no_files_match() {
    let project_root = create_cli_project("json-empty-inputs", &[]);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["check", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["fileCount"], 0);
    assert_eq!(json["errorCount"], 0);
    assert_eq!(json["warningCount"], 0);
    assert_eq!(json["files"].as_array().unwrap().len(), 0);

    let _ = std::fs::remove_dir_all(&project_root);
}
