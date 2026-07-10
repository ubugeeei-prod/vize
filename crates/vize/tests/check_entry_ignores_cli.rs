use std::{path::Path, process::Command};

use vize_carton::cstr;

#[test]
fn check_config_entry_ignores_explicit_inputs() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
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
        .env("CORSA_PATH", corsa_path)
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

#[test]
fn check_config_entry_ignores_default_and_explicit_path_matrix() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "entry-ignore-path-matrix",
        &[
            ("vize.config.json", "{}"),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
const count: number = 1;
</script>
"#,
            ),
            (
                "src/AbsoluteIgnored.vue",
                r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
            ),
            (
                "src/RelativeIgnored.vue",
                r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
            ),
            (
                "src/WindowsIgnored.vue",
                r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
            ),
        ],
    );
    let absolute_ignored = project_root.join("src/AbsoluteIgnored.vue");
    let absolute_ignored_json =
        serde_json::to_string(absolute_ignored.to_string_lossy().as_ref()).unwrap();
    std::fs::write(
        project_root.join("vize.config.json"),
        format!(
            r#"{{
  "entries": [
    {{
      "name": "app",
      "files": ["src/**/*.vue"],
      "ignores": [{absolute_ignored_json}, "src/RelativeIgnored.vue", "src\\WindowsIgnored.vue"]
    }}
  ]
}}"#
        ),
    )
    .unwrap();

    let all = run_check_json(
        &project_root,
        &corsa_path,
        &["check", "--config", "vize.config.json", "--format", "json"],
    );
    assert_json_files(&all, &[("src/App.vue", 0)]);
    assert_eq!(all["errorCount"], 0);

    let ignored_inputs = [
        absolute_ignored.to_string_lossy().into_owned(),
        "src/RelativeIgnored.vue".to_owned(),
        "src/WindowsIgnored.vue".to_owned(),
    ];
    for ignored_input in ignored_inputs {
        let json = run_check_json(
            &project_root,
            &corsa_path,
            &[
                "check",
                "--config",
                "vize.config.json",
                ignored_input.as_str(),
                "--format",
                "json",
            ],
        );
        assert_json_files(&json, &[]);
        assert_eq!(json["errorCount"], 0);
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn run_check_json(project_root: &Path, corsa_path: &str, args: &[&str]) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args(args)
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
    json
}

fn assert_json_files(json: &serde_json::Value, expected: &[(&str, usize)]) {
    let files = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["file"].as_str().unwrap().to_owned(),
                file["diagnostics"].as_array().unwrap().len(),
            )
        })
        .collect::<Vec<_>>();
    let expected = expected
        .iter()
        .map(|(file, diagnostics)| ((*file).to_owned(), *diagnostics))
        .collect::<Vec<_>>();

    assert_eq!(files, expected);
    assert_eq!(json["fileCount"], expected.len());
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn link_workspace_node_modules(project_root: &Path) {
    let source = workspace_root().join("node_modules");
    if !source.exists() {
        return;
    }
    let target = project_root.join("node_modules");
    if target.exists() {
        return;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

fn resolve_test_corsa_path() -> Option<String> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = std::path::PathBuf::from(path);
        if path.exists() {
            return Some(path.display().to_string());
        }
    }

    let workspace_root = workspace_root();
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.display().to_string());
    }

    let workspace_bin = workspace_root.join("node_modules/.bin/tsgo");
    workspace_bin
        .exists()
        .then(|| workspace_bin.display().to_string())
}

fn create_cli_project(name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
    let project_root = std::env::temp_dir()
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}", std::process::id()).as_str());
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    link_workspace_node_modules(&project_root);
    for (path, content) in files {
        let file_path = project_root.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file_path, content).unwrap();
    }
    project_root
}
