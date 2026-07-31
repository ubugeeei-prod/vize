use std::fs;

use super::{VirtualProject, snapshot_text, unique_case_dir};

const MODERN_REF_UNWRAP_HELPER: &str =
    "type __U<T> = T extends import('vue').Ref ? __VizeWidenTemplateRef<T['value']> : T;";
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
    assert!(v2_content.contains("type __VizeVue2ComponentInstance = {\n  $el: Element;"));
    assert!(!default_content.contains("__VizeVue2ComponentInstance"));
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
        assert!(content.contains("type __VizeVue2ComponentInstance = {\n  $el: Element;"));
        assert!(content.contains("  $refs: Record<string, any>;"));
        assert!(content.contains("} & __VizeVue2ComponentInstance;"));
        assert!(!content.contains("import('vue').Ref"));
        assert!(!content.contains("import('vue').ShallowRef"));
        assert!(!content.contains("import('vue').ComponentPublicInstance"));
        assert!(!content.contains("import('vue').defineComponent"));
    }
    assert!(options_content.contains(
        "declare function __vizeDefineComponent<T>(options: T & __VizeNuxt2PageOptions): T;",
    ));

    project.materialize().unwrap();
    assert!(!project.virtual_root().join("__vize_helpers.d.ts").exists());

    let _ = fs::remove_dir_all(&case_dir);
}

/// Type declarations that take every input as a type parameter and are therefore
/// byte-identical in every generated module.
///
/// They belong in the hoisted ambient helpers file, exactly once per program.
/// Declaring them once per generated module costs bytes in every `.vue.ts` *and*
/// hands TypeScript a distinct declaration to instantiate for each one, which
/// defeats its instantiation cache — the effect measured in #3443 and bisected in
/// #3460.
const PROGRAM_WIDE_TEMPLATE_HELPERS: &[&str] =
    &["type __VizeIsUnion<", "type __VizeWidenTemplateRef<"];

fn ref_sfc(binding: &str) -> vize_carton::String {
    vize_carton::cstr!(
        "<script setup lang=\"ts\">
import {{ ref }} from 'vue'
const {binding} = ref('')
</script>

<template>
  <p>{{{{ {binding} }}}}</p>
</template>
"
    )
}

#[test]
fn template_ref_widening_helpers_are_hoisted_once_per_program() {
    let case_dir = unique_case_dir("shared-helper-hoisting");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    let mut paths = Vec::new();
    for binding in ["alpha", "beta", "gamma"] {
        let path = src_dir.join(vize_carton::cstr!("{binding}.vue").as_str());
        fs::write(&path, ref_sfc(binding).as_str()).unwrap();
        paths.push(path);
    }
    project.register_paths(&paths).unwrap();

    for path in &paths {
        let content = project.find_by_original(path).unwrap().content.clone();
        for declaration in PROGRAM_WIDE_TEMPLATE_HELPERS {
            assert_eq!(
                content.matches(declaration).count(),
                0,
                "{} must not declare {declaration} — it is hoisted",
                path.display()
            );
        }
        assert_eq!(
            content.matches(MODERN_REF_UNWRAP_HELPER).count(),
            1,
            "{} must still declare its own dialect-specific __U",
            path.display()
        );
    }

    for declaration in PROGRAM_WIDE_TEMPLATE_HELPERS {
        assert_eq!(
            crate::virtual_ts::SHARED_PREAMBLE_DTS
                .matches(declaration)
                .count(),
            1,
            "the hoisted helpers file must declare {declaration} exactly once"
        );
    }

    let _ = fs::remove_dir_all(&case_dir);
}
