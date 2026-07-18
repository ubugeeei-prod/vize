use std::fs;

use super::{VirtualProject, unique_case_dir};

#[test]
fn register_vue_file_reports_hoisted_macro_local_scope_reference() {
    let case_dir = unique_case_dir("macro-local-scope");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("Bad.vue");
    let vue_content = r#"<script setup lang="ts">
const items = []

withDefaults(defineProps<{
  items?: string[]
}>(), { items })
</script>

<template>{{ items.join() }}</template>
"#;

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_vue_file(&vue_path, vue_content).unwrap();

    let diagnostics = project.diagnostics();
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("SCRIPT_SETUP_MACRO_SCOPE"))
        .expect("typechecker should surface the SFC macro scope diagnostic");
    assert_eq!((diagnostic.line, diagnostic.column), (5, 8));
    assert!(diagnostic.message.contains("`withDefaults()`"));
    assert!(diagnostic.message.contains("`items`"));

    let _ = fs::remove_dir_all(&case_dir);
}
