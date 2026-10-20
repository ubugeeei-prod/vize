//! An imported SFC keeps its required props when passed to Vue's `h()` from a
//! TypeScript module (#9855).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn imported_sfc_h_calls_check_missing_and_wrong_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "component-render-h-props",
        &[
            (
                "src/Counter.vue",
                r#"<script setup lang="ts">
defineProps<{ count: number; label?: string }>();
</script>
<template><span>{{ label }}: {{ count }}</span></template>
"#,
            ),
            (
                "src/render.ts",
                r#"import { h } from "vue";
import Counter from "./Counter.vue";

export const missing = h(Counter, {});
export const wrong = h(Counter, { count: "three" });
export const valid = h(Counter, { count: 3 });
"#,
            ),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };
    assert!(snapshot.is_empty(), "capture exact differential diagnostics: {snapshot:#?}");

    let render_errors: Vec<_> = snapshot
        .iter()
        .filter(|(file, code, _)| file == "src/render.ts" && *code == Some(2769))
        .collect();
    assert_eq!(
        render_errors.len(),
        2,
        "missing and wrong props must each report TS2769 while the valid call remains accepted: {snapshot:#?}"
    );
    assert!(
        render_errors
            .iter()
            .any(|(_, _, message)| message.starts_with("4:"))
            && render_errors
                .iter()
                .any(|(_, _, message)| message.starts_with("5:")),
        "the two diagnostics must map to the authored h() calls: {snapshot:#?}"
    );
}
