use super::*;

#[test]
fn batch_type_checker_reports_camel_case_child_component_prop_error() {
    // Repro 15 from ushironoko/vize-config-repro: `__VizeComponentProps<T>`
    // is a camel/kebab union, so prop value extraction must distribute over
    // the union instead of using `keyof` on the union as a whole.
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "camel-case-child-component-props",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{
  countTotal: number
}>()
</script>

<template>
  <span>{{ countTotal }}</span>
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from "./Child.vue";

const wrong: string = "not a number";
</script>

<template>
  <Child :countTotal="wrong" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot
            .iter()
            .any(|(file, code, message)| file == "src/Parent.vue"
                && *code == Some(2322)
                && message.contains("Type 'string' is not assignable to type 'number'")),
        "expected camelCase child prop mismatch to report TS2322: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_forwarded_optional_component_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "optional-component-props",
        &[
            (
                "src/Provider.vue",
                r#"<script lang="ts">
export type LinkBehavior = "window" | "browser" | null;
</script>

<script setup lang="ts">
defineProps<{
  behavior?: LinkBehavior;
}>();
</script>

<template>
  <a><slot /></a>
</template>
"#,
            ),
            (
                "src/Consumer.vue",
                r#"<script setup lang="ts">
import Provider from "./Provider.vue";
import type { LinkBehavior } from "./Provider.vue";

defineProps<{
  behavior?: LinkBehavior;
}>();
</script>

<template>
  <Provider :behavior="behavior" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "forwarded optional component prop should type-check, got: {snapshot:?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
