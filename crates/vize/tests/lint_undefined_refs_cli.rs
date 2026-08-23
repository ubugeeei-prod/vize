use std::{fs, path::Path, process::Command};

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

fn output_details(output: &std::process::Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn lint_config_can_enable_undefined_template_refs() {
    let project_root = tempfile::tempdir().unwrap();
    write_file(
        project_root.path(),
        "vize.config.json",
        r#"{
  "linter": {
    "rules": {
      "vue/no-undefined-refs": "warn"
    }
  }
}"#,
    );
    write_file(
        project_root.path(),
        "src/App.vue",
        r#"<script setup>
const known = 1
</script>
<template>
  <p>{{ known }} {{ missing }}</p>
</template>
"#,
    );

    let enabled = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "src/App.vue",
        ])
        .output()
        .unwrap();
    let enabled_stdout = String::from_utf8_lossy(&enabled.stdout);
    assert!(
        enabled_stdout.contains("vue/no-undefined-refs"),
        "config-enable must report the undefined template ref: {}",
        output_details(&enabled)
    );
    assert!(
        enabled_stdout.contains("missing"),
        "{}",
        output_details(&enabled)
    );

    let default = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root.path())
        .args(["lint", "--no-config", "--format", "json", "src/App.vue"])
        .output()
        .unwrap();
    let default_stdout = String::from_utf8_lossy(&default.stdout);
    assert!(
        !default_stdout.contains("vue/no-undefined-refs"),
        "default preset must stay silent: {}",
        output_details(&default)
    );
}
