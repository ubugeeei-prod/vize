//! Runtime Vue APIs require imports even with the workspace fallback SDK.
use super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};
use std::path::Path;
use vize_carton::String;

fn snapshot(root: &Path) -> Vec<(String, Option<u32>, String)> {
    snapshot_project_diagnostics(root).expect("the available native backend must run")
}

fn vector(rows: &[(String, Option<u32>, String)]) -> Vec<(&str, Option<u32>)> {
    let mut result = rows
        .iter()
        .map(|(file, code, _)| (file.as_str(), *code))
        .collect::<Vec<_>>();
    result.sort_unstable();
    result
}

#[test]
fn workspace_vue_runtime_preserves_missing_import_then_repair() {
    let Some(backend) = resolve_test_tsgo_binary() else {
        return;
    };
    assert!(backend.is_file(), "the resolved native backend must exist");
    let valid_import = r#"<script setup lang="ts">
import { useTemplateRef } from 'vue'
const element = useTemplateRef<HTMLInputElement>('input')
if (element.value) {
  const numeric: number = element.value.value
  element.value.noSuchOriginalMember()
}
</script>
<template><input ref="input" /></template>
"#;
    let missing_import = r#"<script setup lang="ts">
const missing = useTemplateRef<HTMLInputElement>('input')
</script>
<template><input ref="input" /></template>
"#;
    let root = create_project_case_without_node_modules(
        "template-ref-import-admission",
        &[
            ("src/TypedElement.vue", valid_import),
            ("src/MissingImport.vue", missing_import),
        ],
    );
    let first = snapshot(&root);
    assert_eq!(
        vector(&first),
        vec![
            ("src/MissingImport.vue", Some(2304)),
            ("src/TypedElement.vue", Some(2322)),
            ("src/TypedElement.vue", Some(2339)),
        ],
        "every original error must survive: {first:#?}"
    );
    assert_eq!(
        first
            .iter()
            .find(|(file, _, _)| file == "src/MissingImport.vue")
            .unwrap()
            .2,
        "2:17:error Cannot find name 'useTemplateRef'."
    );
    let repaired = missing_import.replacen(
        "<script setup lang=\"ts\">\n",
        "<script setup lang=\"ts\">\nimport { useTemplateRef } from 'vue'\n",
        1,
    );
    std::fs::write(root.join("src/MissingImport.vue"), repaired).unwrap();
    let second = snapshot(&root);
    assert_eq!(
        vector(&second),
        vec![
            ("src/TypedElement.vue", Some(2322)),
            ("src/TypedElement.vue", Some(2339)),
        ],
        "import repair removes only its own diagnostic: {second:#?}"
    );
    let _ = std::fs::remove_dir_all(root);
}
