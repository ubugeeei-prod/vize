use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn script_setup_import_meta_uses_project_import_meta_type() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "script-setup-import-meta-type",
        &[
            (
                "src/env.d.ts",
                r#"interface ImportMeta {
  readonly __vizeTestEnv: { readonly MODE: string }
  __vizeTestGlob<T>(pattern: string): Record<string, () => Promise<T>>
}
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
const modules = import.meta.__vizeTestGlob<{ default: object }>("./*.vue")
const mode: number = import.meta.__vizeTestEnv.MODE
</script>
<template><p>{{ Object.keys(modules).length }} {{ mode }}</p></template>
"#,
            ),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    assert_eq!(
        snapshot,
        Some(vec![(
            vize_l0::String::from("src/App.vue"),
            Some(2322),
            vize_l0::String::from("3:7:error Type 'string' is not assignable to type 'number'."),
        )]),
    );
}
