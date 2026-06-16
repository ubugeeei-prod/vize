use std::fs;

use super::{VirtualProject, snapshot_text, unique_case_dir};

#[test]
fn configured_dialect_drives_dialect_aware_instance_typing() {
    // Issue #1392/#1802: Vue 2.7 augments template context and uses
    // single-arg Ref<T> matching, while default Vue 3 keeps Ref<T, S>.
    let vue_content = r#"<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  test: string
}>()

const count = ref(0)
</script>

<template>
  <div>{{ test }} {{ count }}</div>
</template>
"#;

    let v3_dir = unique_case_dir("dialect-default-v3");
    let _ = fs::remove_dir_all(&v3_dir);
    let v3_src = v3_dir.join("src");
    fs::create_dir_all(&v3_src).unwrap();
    let v3_path = v3_src.join("App.vue");
    fs::write(&v3_path, vue_content).unwrap();

    let mut default_project = VirtualProject::new(&v3_dir).unwrap();
    default_project.register_path(&v3_path).unwrap();
    let default_content = default_project
        .find_by_original(&v3_path)
        .unwrap()
        .content
        .clone();

    let v2_dir = unique_case_dir("dialect-v2-7");
    let _ = fs::remove_dir_all(&v2_dir);
    let v2_src = v2_dir.join("src");
    fs::create_dir_all(&v2_src).unwrap();
    let v2_path = v2_src.join("App.vue");
    fs::write(&v2_path, vue_content).unwrap();

    let mut v2_project = VirtualProject::new(&v2_dir).unwrap();
    v2_project.set_dialect(vize_carton::config::VueVersion::V2_7);
    v2_project.register_path(&v2_path).unwrap();
    let v2_content = v2_project
        .find_by_original(&v2_path)
        .unwrap()
        .content
        .clone();

    insta::assert_snapshot!(
        "dialect_default_v3",
        snapshot_text(default_content.as_str())
    );
    insta::assert_snapshot!("dialect_v2", snapshot_text(v2_content.as_str()));
    assert!(default_content.contains("Ref<infer V, any>"));
    assert!(v2_content.contains("Ref<infer V>"));
    assert!(!v2_content.contains("Ref<infer V, any>"));
    assert_ne!(v2_content, default_content);

    let _ = fs::remove_dir_all(&v3_dir);
    let _ = fs::remove_dir_all(&v2_dir);
}

#[test]
fn legacy_vue2_ref_unwrap_uses_vue2_ref_arity() {
    // Issue #1802: `typeChecker.legacyVue2` must not emit Vue 3.4+ Ref<T, S>.
    let case_dir = unique_case_dir("legacy-vue2-ref-unwrapped");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let vue_path = src_dir.join("App.vue");
    let vue_content = r#"<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)
</script>

<template>
  <span>{{ count }}</span>
</template>
"#;
    fs::write(&vue_path, vue_content).unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.set_legacy_vue2(true);
    project.register_path(&vue_path).unwrap();
    let content = project.find_by_original(&vue_path).unwrap().content.clone();

    assert!(content.contains("Ref<infer V>"));
    assert!(!content.contains("Ref<infer V, any>"));

    let _ = fs::remove_dir_all(&case_dir);
}
