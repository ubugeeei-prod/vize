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

    assert_eq!(
        snapshot,
        vec![
            (
                "src/render.ts".to_string(),
                Some(2769),
                "4:24:error capture exact TS2769 missing-props diagnostic".to_string(),
            ),
            (
                "src/render.ts".to_string(),
                Some(2769),
                "5:22:error capture exact TS2769 wrong-props diagnostic".to_string(),
            ),
        ],
        "only the missing and wrong h() calls may report diagnostics: {snapshot:#?}"
    );
}
