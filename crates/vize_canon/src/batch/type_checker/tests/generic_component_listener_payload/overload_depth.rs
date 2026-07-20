use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn generic_emit_overload_expansion_stays_bounded() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-emit-overload-depth",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="T extends keyof Payloads = any">
export interface Payloads {
  text: string
  count: number
}

defineProps<{ type: T }>()
defineEmits<{
  <K extends keyof Payloads>(event: "done", value: Payloads[K]): void
  (event: "closed"): void
}>()
</script>

<template><div /></template>
"#,
            ),
            (
                "src/usage.ts",
                r#"import Child from "./Child.vue";

type StaticEventProps<C> = C extends { __vizeEmitProps?: infer P }
  ? NonNullable<P>
  : never;
type ExpandedEventProps = {
  [K in keyof StaticEventProps<typeof Child>]: StaticEventProps<typeof Child>[K]
};

declare const handlers: ExpandedEventProps;
handlers.onClosed?.();
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(_, code, _)| *code != Some(2589)),
        "generic emit prop expansion should not exceed the checker depth: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
