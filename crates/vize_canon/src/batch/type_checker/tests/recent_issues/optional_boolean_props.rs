//! Optional boolean props are normalized when passed through templates (#2719).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn issue_2719_normalizes_optional_boolean_props_in_template() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "issue-2719-optional-boolean-template-prop",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ present: boolean }>();
</script>

<template>
  <span />
</template>
"#,
            ),
            (
                "src/Foo.vue",
                r#"<script setup lang="ts">
import Child from "./Child.vue";

defineProps<{ loading?: boolean }>();
</script>

<template>
  <Child :present="loading" />
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
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Foo.vue"
                && *code == Some(2322)
                && message.contains("boolean | undefined"))
        }),
        "optional boolean props should be normalized in templates, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn with_defaults_preserves_optional_boolean_define_props_narrowing() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "with-defaults-optional-boolean-props",
        &[(
            "src/Foo.vue",
            r#"<script setup lang="ts">
import { computed } from "vue";
import type { ComputedRef } from "vue";

interface Props {
  disabled?: boolean;
  unmountOnHide?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  unmountOnHide: undefined,
});
const disabled = computed(() => props.disabled);

function acceptDisabled(value: ComputedRef<boolean>) {
  return value;
}

acceptDisabled(disabled);
</script>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Foo.vue"
                && *code == Some(2345)
                && message.contains("ComputedRef<boolean | undefined>"))
        }),
        "withDefaults should keep optional boolean defineProps reads narrowed, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn with_defaults_preserves_optional_boolean_props_for_overload_resolution() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "with-defaults-optional-boolean-overloads",
        &[(
            "src/Foo.vue",
            r#"<script setup lang="ts" generic="T">
interface Props<T> {
  defaultValue?: T;
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props<T>>(), {
  defaultValue: undefined,
});

declare function acceptState(value: { disabled: true; defaultValue?: T }): void;
declare function acceptState(value: { disabled: false; defaultValue?: T }): void;
declare function acceptState(value: { disabled: boolean; defaultValue?: T }): void;

acceptState({
  defaultValue: props.defaultValue,
  disabled: props.disabled,
});
</script>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Foo.vue"
                && *code == Some(2769)
                && message.contains("No overload matches this call"))
        }),
        "withDefaults should keep optional boolean props usable by overloads, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
