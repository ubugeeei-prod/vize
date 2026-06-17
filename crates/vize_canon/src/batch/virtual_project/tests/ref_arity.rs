use std::fs;

use super::{VirtualProject, snapshot_text, unique_case_dir};

const MODERN_REF_UNWRAP_HELPER: &str =
    "type __U<T> = T extends import('vue').Ref ? T['value'] : T;";
const LEGACY_REF_UNWRAP_HELPER: &str = "type __U<T> = T extends { value: infer __V } ? __V : T;";

#[test]
fn configured_dialect_drives_instance_typing_without_ref_arity() {
    // Issue #1392/#1802: Vue 2.7 augments template context, but ref unwrapping
    // must follow the user's installed Vue types instead of encoding an arity.
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
    assert!(default_content.contains(MODERN_REF_UNWRAP_HELPER));
    assert!(v2_content.contains(LEGACY_REF_UNWRAP_HELPER));
    assert!(!default_content.contains("Ref<infer"));
    assert!(!v2_content.contains("Ref<infer"));
    assert!(v2_content.contains("$listeners"));
    assert!(!default_content.contains("$listeners"));
    assert!(!v2_content.contains("Ref<infer V, any>"));
    assert!(!v2_content.contains("import('vue').Ref"));
    assert!(!v2_content.contains("import('vue').ComponentPublicInstance"));
    assert_ne!(v2_content, default_content);

    let _ = fs::remove_dir_all(&v3_dir);
    let _ = fs::remove_dir_all(&v2_dir);
}

#[test]
fn legacy_vue2_ref_unwrap_uses_installed_vue_types() {
    // Issue #1802: `typeChecker.legacyVue2` must not emit Vue-version-specific
    // Ref<T, S> or Ref<T> arity in virtual TS.
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

    assert!(content.contains(LEGACY_REF_UNWRAP_HELPER));
    assert!(!content.contains("Ref<infer"));
    assert!(!content.contains("Ref<infer V, any>"));
    assert!(!content.contains("import('vue').Ref"));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn legacy_vue2_virtual_ts_avoids_vue3_only_helper_exports() {
    let case_dir = unique_case_dir("legacy-vue2-no-vue3-helpers");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let setup_path = src_dir.join("Setup.vue");
    let options_path = src_dir.join("Options.vue");
    fs::write(
        &setup_path,
        r#"<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'

const count = ref(0)
const input = useTemplateRef<HTMLInputElement>('input')
</script>

<template>
  <span>{{ count }} {{ input }}</span>
</template>
"#,
    )
    .unwrap();
    fs::write(
        &options_path,
        r#"<script lang="ts">
export default {
  data() {
    return { count: 0 }
  },
}
</script>

<template>
  <span>{{ count }}</span>
</template>
"#,
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.set_legacy_vue2(true);
    project.register_path(&setup_path).unwrap();
    project.register_path(&options_path).unwrap();

    let setup_content = project
        .find_by_original(&setup_path)
        .unwrap()
        .content
        .clone();
    let options_content = project
        .find_by_original(&options_path)
        .unwrap()
        .content
        .clone();
    assert!(setup_content.contains(LEGACY_REF_UNWRAP_HELPER));
    for content in [setup_content.as_str(), options_content.as_str()] {
        assert!(!content.contains("import('vue').Ref"));
        assert!(!content.contains("import('vue').ShallowRef"));
        assert!(!content.contains("import('vue').ComponentPublicInstance"));
        assert!(!content.contains("import('vue').defineComponent"));
    }
    assert!(options_content.contains("declare function __vizeDefineComponent<T>(options: T): T;"));

    project.materialize().unwrap();
    assert!(!project.virtual_root().join("__vize_helpers.d.ts").exists());

    let _ = fs::remove_dir_all(&case_dir);
}
