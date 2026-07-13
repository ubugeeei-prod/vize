use super::*;

#[test]
fn normalizes_imported_inherited_boolean_prop_in_generic_setup() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    crate::virtual_ts::reset_authored_script_fallback_parse_invocations();
    let project_root = create_project_case(
        "issue-2806-generic-inherited-boolean-prop",
        &[
            (
                "src/base.ts",
                r#"export interface BaseProps {
  disabled?: boolean;
}
"#,
            ),
            (
                "src/Foo.vue",
                r#"<script setup lang="ts" generic="T">
import { computed } from "vue";
import type { BaseProps } from "./base";

interface FooProps extends BaseProps {
  value?: T;
}

const props = defineProps<FooProps>();

function takesBoolean(value: boolean): boolean {
  return value;
}

const disabled = computed(() => takesBoolean(props.disabled));
</script>

<template>
  <div>{{ disabled }}</div>
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
        "imported inherited Boolean props must normalize to boolean: {snapshot:#?}"
    );
    assert_eq!(
        crate::virtual_ts::authored_script_fallback_parse_invocations(),
        0,
        "imported Boolean-key fallback must come from parse-once SFC facts",
    );
    let _ = std::fs::remove_dir_all(&project_root);
}
