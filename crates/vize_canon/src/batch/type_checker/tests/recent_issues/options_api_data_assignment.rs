//! Options API `data()` members stay mutable in template assignment expressions.

use std::path::Path;

use super::super::{
    BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::TypeChecker;

type DiagnosticRow = (
    std::string::String,
    Option<u32>,
    u32,
    u32,
    u8,
    std::string::String,
);

const DATA_ASSIGNMENT_SFC: &str = r#"<template>
    <button @click="open = true">open</button>
    <button @click="open = false">close</button>
    <span>{{ open }}</span>
</template>
<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
    data() {
        return {
            open: false,
        }
    },
})
</script>
"#;

#[test]
fn options_api_data_assignments_are_mutable_from_template_expressions() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "options-api-data-template-assignment",
        &[("src/OptionsApi.vue", DATA_ASSIGNMENT_SFC)],
    );
    if !project_root.join("node_modules/vue/dist").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    assert_eq!(
        project_diagnostics(&project_root),
        Vec::<DiagnosticRow>::new(),
        "`data()` properties are mutable through the Options API instance"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn project_diagnostics(project_root: &Path) -> Vec<DiagnosticRow> {
    let mut checker = BatchTypeChecker::new(project_root).expect("batch checker construction");
    checker.enable_options_api();
    checker.scan_project().expect("project scan");
    let result = checker.check_project().expect("project check");
    let mut rows: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file).to_string(),
                diagnostic.code,
                diagnostic.line + 1,
                diagnostic.column + 1,
                diagnostic.severity,
                diagnostic.message.to_string(),
            )
        })
        .collect();
    rows.sort();
    rows
}
