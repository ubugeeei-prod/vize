use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-lint-config-cli-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

#[test]
fn lint_uses_entry_preset_unless_cli_preset_overrides() {
    let project_root = temp_project_dir("entry-preset");
    write_project_file(
        &project_root,
        "vize.config.json",
        r#"{
  "entries": [
    {
      "name": "app",
      "files": ["src/**/*.vue"],
      "linter": { "preset": "incremental" }
    }
  ]
}"#,
    );
    write_project_file(
        &project_root,
        "src/App.vue",
        r#"<script setup>
const noop = () => {}
</script>

<template>
  <div @click="noop">Clickable</div>
</template>
"#,
    );

    let configured = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--config", "vize.config.json", "src/App.vue"])
        .output()
        .unwrap();
    assert!(
        configured.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&configured.stdout),
        String::from_utf8_lossy(&configured.stderr)
    );
    let configured_stdout = String::from_utf8_lossy(&configured.stdout);
    assert!(!configured_stdout.contains("a11y/click-events-have-key-events"));

    let overridden = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--preset",
            "ecosystem",
            "--max-warnings",
            "0",
            "src/App.vue",
        ])
        .output()
        .unwrap();
    assert!(
        !overridden.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&overridden.stdout),
        String::from_utf8_lossy(&overridden.stderr)
    );
    let stdout = String::from_utf8_lossy(&overridden.stdout);
    assert!(stdout.contains("a11y/click-events-have-key-events"));

    let _ = fs::remove_dir_all(project_root);
}
