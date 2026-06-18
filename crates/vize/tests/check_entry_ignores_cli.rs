use std::{path::Path, process::Command};

use vize_carton::cstr;

#[test]
fn check_config_entry_ignores_explicit_inputs() {
    let project_root = create_cli_project(
        "entry-ignore-explicit-check",
        &[
            (
                "vize.config.json",
                r#"{
  "entries": [
    { "name": "app", "files": ["src/**/*.vue"], "ignores": ["src/Ignored.vue"] }
  ]
}"#,
            ),
            (
                "src/Ignored.vue",
                r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
            ),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "check",
            "--config",
            "vize.config.json",
            "src/Ignored.vue",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });

    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["fileCount"], 0);
    assert_eq!(json["errorCount"], 0);
    assert_eq!(json["files"].as_array().unwrap().len(), 0);

    let _ = std::fs::remove_dir_all(&project_root);
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn create_cli_project(name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str());
    let _ = std::fs::remove_dir_all(&project_root);
    for (path, content) in files {
        let file_path = project_root.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file_path, content).unwrap();
    }
    project_root
}
