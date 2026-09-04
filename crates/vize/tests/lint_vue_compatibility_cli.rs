use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn temp_project_dir() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-lint-vue-compatibility-{}-{nonce}",
        std::process::id()
    ))
}

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

fn output_details(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn lint_vue2_compatibility_disables_vue34_props_shorthand() {
    let project_root = temp_project_dir();
    write_project_file(
        &project_root,
        "src/App.vue",
        r#"<script setup>
const title = 'legacy'
</script>

<template>
  <LegacyCard :title="title" />
</template>
"#,
    );
    write_project_file(
        &project_root,
        "vize.config.json",
        r#"{ "compiler": { "compatibility": { "vueVersion": "2.7" } } }"#,
    );

    let vue3 = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--preset", "nuxt", "--no-config", "src/App.vue"])
        .output()
        .unwrap();
    assert!(vue3.status.success(), "{}", output_details(&vue3));
    assert!(
        String::from_utf8_lossy(&vue3.stdout).contains("vue/prefer-props-shorthand"),
        "{}",
        output_details(&vue3)
    );

    let vue2 = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--preset",
            "nuxt",
            "--config",
            "vize.config.json",
            "src/App.vue",
        ])
        .output()
        .unwrap();
    assert!(vue2.status.success(), "{}", output_details(&vue2));
    let vue2_stdout = String::from_utf8_lossy(&vue2.stdout);
    assert!(vue2_stdout.contains("No problems found"), "{vue2_stdout}");
    assert!(
        !vue2_stdout.contains("vue/prefer-props-shorthand"),
        "{vue2_stdout}"
    );

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn lint_top_level_vue2_compatibility_disables_vue34_props_shorthand() {
    let project_root = temp_project_dir();
    write_project_file(
        &project_root,
        "src/DataCard.vue",
        r#"<script setup lang="ts">
interface Props {
  height?: string
}
withDefaults(defineProps<Props>(), { height: '' })
</script>

<template>
  <v-card :height="height">sample</v-card>
</template>
"#,
    );
    write_project_file(
        &project_root,
        "vize.config.ts",
        r#"export default {
  compatibility: { vueVersion: '2.7' }
}
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "lint",
            "--preset",
            "opinionated",
            "--config",
            "vize.config.ts",
            "src/DataCard.vue",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", output_details(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("vue/prefer-props-shorthand"), "{stdout}");

    let _ = fs::remove_dir_all(project_root);
}
