use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use vize_s0::cstr;

fn temp_project_dir() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(
        cstr!(
            "vize-lint-rule-options-casing-{}-{nonce}",
            std::process::id()
        )
        .as_str(),
    )
}

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

fn output_details(output: &Output) -> vize_s0::String {
    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("<non-utf8 stdout>");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>");
    cstr!("stdout:\n{}\nstderr:\n{}", stdout, stderr)
}

#[test]
fn lint_rule_options_configure_template_and_event_casing() {
    let project_root = temp_project_dir();
    write_project_file(
        &project_root,
        "src/Casing.vue",
        r#"<template>
  <my-widget />
  <MyWidget />
</template>

<script setup lang="ts">
import MyWidget from './MyWidget.vue';

const emit = defineEmits<{ 'keep-original': []; keepOriginal: [] }>();

emit('keep-original');
emit('keepOriginal');
</script>
"#,
    );
    write_project_file(
        &project_root,
        "vize.config.json",
        r#"{
  "linter": {
    "preset": "incremental",
    "rules": {
      "vue/component-name-in-template-casing": "error",
      "script/custom-event-name-casing": "error"
    },
    "ruleOptions": {
      "vue/component-name-in-template-casing": { "casing": "kebab-case" },
      "script/custom-event-name-casing": { "casing": "kebab-case" }
    }
  }
}"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--config",
            "vize.config.json",
            "--format",
            "json",
            "src/Casing.vue",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success(), "{}", output_details(&output));

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        stdout.contains("vue/component-name-in-template-casing"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Component should use kebab-case"),
        "{stdout}"
    );
    assert!(
        stdout.contains("script/custom-event-name-casing"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Custom event name 'keepOriginal' is not kebab-case."),
        "{stdout}"
    );
    assert!(
        !stdout.contains("Custom event name 'keep-original' is not"),
        "{stdout}"
    );

    let _ = fs::remove_dir_all(project_root);
}
