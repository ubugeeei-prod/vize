use super::{Command, create_cli_project};

#[test]
fn check_json_reports_empty_result_when_no_files_match() {
    // An out-of-repository empty directory cannot inherit the workspace's tsconfig.
    let case = super::unique_case_dir("json-empty-inputs");
    let project_root = std::env::temp_dir().join(case.file_name().unwrap());
    std::fs::create_dir(&project_root).unwrap();

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
    assert_eq!(stderr, "");
    assert_eq!(
        json,
        serde_json::json!({
            "files": [], "programs": [], "errorCount": 0,
            "warningCount": 0, "fileCount": 0
        })
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_json_refuses_discovered_zero_workload_project() {
    let project_root = create_cli_project("json-selected-empty-inputs", &[]);
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["check", "--format", "json", "--quiet"])
        .output()
        .unwrap();
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(output.status.code(), Some(2), "{stdout}\n{stderr}");
    assert_eq!(
        json,
        serde_json::json!({
            "files": [], "programs": [], "errorCount": 0,
            "warningCount": 0, "fileCount": 0
        })
    );
    assert_eq!(
        stderr,
        format!(
            "Error: No supported source files were selected by TypeScript project `{}`; no files were checked. Check the project's files/include/exclude and Vize ignores.\n",
            project_root.join("tsconfig.json").display()
        )
    );
    let _ = std::fs::remove_dir_all(&project_root);
}
