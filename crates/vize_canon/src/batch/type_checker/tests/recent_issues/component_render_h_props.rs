//! An imported SFC keeps its required props when passed to Vue's `h()` from a
//! TypeScript module (#9855).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

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

    let render_errors: Vec<_> = snapshot
        .iter()
        .filter(|(file, code, _)| file == "src/render.ts" && *code == Some(2769))
        .collect();
    assert_eq!(
        render_errors.len(),
        2,
        "missing and wrong props must each report TS2769 while the valid call remains accepted: {snapshot:#?}"
    );
    let positions: Vec<_> = render_errors
        .iter()
        .map(|(file, code, message)| {
            (
                file.as_str(),
                *code,
                message.split(':').take(2).collect::<Vec<_>>().join(":"),
            )
        })
        .collect();
    assert_eq!(
        positions,
        vec![
            ("src/render.ts", Some(2769), "4:24".to_string()),
            ("src/render.ts", Some(2769), "5:22".to_string()),
        ],
        "the two diagnostics must map to the authored h() calls: {snapshot:#?}"
    );
}
