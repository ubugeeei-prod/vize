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

#[test]
fn lint_vue2_config_allows_template_v_for_key_on_child() {
    let project_root = temp_project_dir("vue2-v-for-child-key");
    let sfc = r#"<script setup>
const items = [{ id: 1, name: 'One' }]
</script>

<template>
  <template v-for="item in items">
    <div :key="item.id">{{ item.name }}</div>
  </template>
</template>
"#;
    write_project_file(&project_root, "src/App.vue", sfc);
    write_project_file(
        &project_root,
        "vize.config.json",
        r#"{ "vue": { "version": "2" } }"#,
    );

    let vue3 = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--no-config", "src/App.vue"])
        .output()
        .unwrap();
    assert!(!vue3.status.success());
    let stdout = String::from_utf8_lossy(&vue3.stdout);
    assert!(stdout.contains("vue/no-v-for-template-key-on-child"));

    let vue2 = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--config", "vize.config.json", "src/App.vue"])
        .output()
        .unwrap();
    assert!(
        vue2.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&vue2.stdout),
        String::from_utf8_lossy(&vue2.stderr)
    );

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn lint_compiler_compatibility_vue2_allows_template_v_for_key_on_child() {
    let project_root = temp_project_dir("compat-vue2-v-for-child-key");
    let sfc = r#"<script setup>
const items = [{ id: 1, name: 'One' }]
</script>

<template>
  <template v-for="item in items">
    <div :key="item.id">{{ item.name }}</div>
  </template>
</template>
"#;
    write_project_file(&project_root, "src/App.vue", sfc);
    write_project_file(
        &project_root,
        "vize.config.json",
        r#"{ "compiler": { "compatibility": { "vueVersion": "2" } } }"#,
    );

    let compat_vue2 = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--config", "vize.config.json", "src/App.vue"])
        .output()
        .unwrap();
    assert!(
        compat_vue2.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compat_vue2.stdout),
        String::from_utf8_lossy(&compat_vue2.stderr)
    );

    write_project_file(
        &project_root,
        "vize.config.json",
        r#"{ "typeChecker": { "legacyVue2": true } }"#,
    );
    let legacy_vue2 = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--config", "vize.config.json", "src/App.vue"])
        .output()
        .unwrap();
    assert!(
        legacy_vue2.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&legacy_vue2.stdout),
        String::from_utf8_lossy(&legacy_vue2.stderr)
    );

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn lint_nuxt_preset_allows_next_tick_by_default_and_when_compiler_vapor_is_false() {
    let project_root = temp_project_dir("nuxt-non-vapor-next-tick");
    let sfc = r#"<script setup lang="ts">
import { nextTick } from 'vue'

await nextTick()
</script>
"#;
    write_project_file(&project_root, "src/Dialog.vue", sfc);
    write_project_file(
        &project_root,
        "vize.config.json",
        r#"{
  "compiler": { "vapor": false },
  "linter": { "preset": "nuxt" }
}"#,
    );

    let nuxt_default = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--preset", "nuxt", "--no-config", "src/Dialog.vue"])
        .output()
        .unwrap();
    assert!(
        nuxt_default.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&nuxt_default.stdout),
        String::from_utf8_lossy(&nuxt_default.stderr)
    );

    let non_vapor = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["lint", "--config", "vize.config.json", "src/Dialog.vue"])
        .output()
        .unwrap();
    assert!(
        non_vapor.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&non_vapor.stdout),
        String::from_utf8_lossy(&non_vapor.stderr)
    );

    let _ = fs::remove_dir_all(project_root);
}
