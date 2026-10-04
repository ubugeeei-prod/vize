//! A prop's declared casing survives the template binding (#3863).
//!
//! Attribute names used to be lowercased on their first character, so a prop
//! literally named `Template` was looked up as `template` and reported missing
//! even though the parent bound it by name. The underscore in a snake_case prop
//! was likewise treated as a separator.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

/// Oracle: vue-tsc 3.3.9 with TypeScript 6.x reports no diagnostics for these
/// files — every prop is bound by its declared name.
#[test]
fn pascal_case_and_snake_case_props_are_not_reported_missing() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "pascal-case-prop-names",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ readonly Template: string; readonly other: number }>()
</script>
<template><span /></template>
"#,
            ),
            (
                "src/SnakeChild.vue",
                r#"<script setup lang="ts">
defineProps<{ readonly my_prop: string }>()
</script>
<template><span /></template>
"#,
            ),
            (
                "src/KebabChild.vue",
                r#"<script setup lang="ts">
defineProps<{ readonly someValue: number }>()
</script>
<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
import SnakeChild from './SnakeChild.vue'
import KebabChild from './KebabChild.vue'
const Template = 'x'
</script>
<template>
  <Child :Template="Template" :other="1" />
  <SnakeChild :my_prop="Template" />
  <KebabChild :some-value="1" />
  <KebabChild :someValue="1" />
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
        vec![],
        "declared prop casing must survive the binding, matching vue-tsc"
    );
}
