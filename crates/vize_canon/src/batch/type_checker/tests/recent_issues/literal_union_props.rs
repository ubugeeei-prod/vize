//! Authored static string props must retain their literal type through the
//! whole-props checker (#3599).
//!
//! Oracle: TypeScript 6.0.3 and Vue 3.6.0-beta.10.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_s0::{String, cstr};

#[test]
fn string_literal_union_props_report_only_the_wrong_literal() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "literal-union-props",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ align: 'start' | 'end' }>()
</script>
<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child align="start" />
  <Child align="middle" />
</template>
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
        vec![(
            String::from("src/Parent.vue"),
            Some(2322),
            cstr!("7:10:error Type '\"middle\"' is not assignable to type '\"end\" | \"start\"'."),
        )],
        "the valid literal stays accepted and the invalid literal produces one complete diagnostic"
    );
}
