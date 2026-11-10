use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn explicit_generic_arguments_specialize_listener_parameters() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "explicit-generic-event-parameters",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T extends string | number, K = any">
defineEmits<{
  foo: [value: T, flag: K]
}>()
</script>
"#,
            ),
            (
                "src/shared.d.ts",
                r#"type IfEquals<T, U, Y = unknown, N = never> =
  (<G>() => G extends T ? 1 : 2) extends (<G>() => G extends U ? 1 : 2) ? Y : N
declare function exactType<T, U>(
  actual: T & IfEquals<T, U>,
  expected: U & IfEquals<T, U>,
): IfEquals<T, U>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <!-- @vue-generic {string} -->
  <Child @foo="(value) => exactType(value, {} as string)" />
  <!-- @vue-generic {number, boolean} -->
  <Child @foo="(value, flag) => (
    exactType(value, {} as number),
    exactType(flag, {} as boolean)
  )" />
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

    assert_eq!(snapshot, Vec::new());
}
