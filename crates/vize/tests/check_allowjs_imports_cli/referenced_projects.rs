use super::*;

#[test]
fn referenced_projects_apply_allowjs_to_explicit_and_default_checks() {
    let Some(corsa_path) = required_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("explicit-referenced-project");
    let _ = std::fs::remove_dir_all(&project_root);

    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "files": [],
  "references": [
    { "path": "./packages/allow" },
    { "path": "./packages/deny" }
  ]
}"#,
    );
    write(
        &project_root,
        "packages/allow/tsconfig.json",
        r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    );
    write(
        &project_root,
        "packages/deny/tsconfig.json",
        r#"{
  "compilerOptions": { "allowJs": false },
  "include": ["src/**/*"]
}"#,
    );
    write(
        &project_root,
        "packages/allow/src/invalid.js",
        r#"/** @type {string} */
export const message = 42;
"#,
    );
    write(
        &project_root,
        "packages/deny/src/ignored.js",
        r#"/** @type {string} */
export const ignored = 42;
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "packages/allow/src/invalid.js",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("invalid JSON ({error}):\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    assert_eq!(json["errorCount"], serde_json::json!(1), "{stdout}");
    assert_eq!(json["fileCount"], serde_json::json!(1), "{stdout}");
    assert_eq!(
        json["files"][0]["file"],
        serde_json::json!("packages/allow/src/invalid.js"),
        "{stdout}"
    );
    assert!(
        json["files"][0]["diagnostics"][0]
            .as_str()
            .is_some_and(|diagnostic| diagnostic.contains("TS2322")),
        "{stdout}"
    );
    assert!(!stdout.contains("ignored.js"), "{stdout}");

    let denied_output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "packages/deny/src/ignored.js",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let denied_stdout = std::string::String::from_utf8(denied_output.stdout).unwrap();
    let denied_stderr = std::string::String::from_utf8(denied_output.stderr).unwrap();
    assert!(
        denied_output.status.success(),
        "stdout:\n{denied_stdout}\nstderr:\n{denied_stderr}"
    );
    let denied_json: serde_json::Value =
        serde_json::from_str(&denied_stdout).unwrap_or_else(|error| {
            panic!("invalid JSON ({error}):\nstdout:\n{denied_stdout}\nstderr:\n{denied_stderr}")
        });
    assert_eq!(
        denied_json["fileCount"],
        serde_json::json!(0),
        "{denied_stdout}"
    );

    let default_output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let default_stdout = std::string::String::from_utf8(default_output.stdout).unwrap();
    let default_stderr = std::string::String::from_utf8(default_output.stderr).unwrap();
    assert_eq!(
        default_output.status.code(),
        Some(1),
        "stdout:\n{default_stdout}\nstderr:\n{default_stderr}"
    );
    let default_json: serde_json::Value =
        serde_json::from_str(&default_stdout).unwrap_or_else(|error| {
            panic!("invalid JSON ({error}):\nstdout:\n{default_stdout}\nstderr:\n{default_stderr}")
        });
    assert_eq!(
        default_json["errorCount"],
        serde_json::json!(1),
        "{default_stdout}"
    );
    assert_eq!(
        default_json["fileCount"],
        serde_json::json!(1),
        "{default_stdout}"
    );
    assert_eq!(
        default_json["files"][0]["file"],
        serde_json::json!("packages/allow/src/invalid.js"),
        "{default_stdout}"
    );
    assert!(!default_stdout.contains("ignored.js"), "{default_stdout}");

    let _ = std::fs::remove_dir_all(&project_root);
}
