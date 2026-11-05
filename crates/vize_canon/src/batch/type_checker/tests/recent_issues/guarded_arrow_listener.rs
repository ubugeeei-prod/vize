//! A callable fallback inside a guarded arrow listener must not make the
//! render-time guard appear to test an always-defined function (#11965).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

fn diagnostics(listener: &str) -> Option<Vec<(vize_l0::String, Option<u32>, vize_l0::String)>> {
    let probe = format!(
        r#"<script setup lang="ts">
import Child from './Child.vue';
defineProps<{{ loading?: boolean; actions?: Map<string, boolean>; handler?: (action: string) => void }}>();
</script>

<template>
  <div v-if="actions && handler">
    <Child v-if="actions && handler && actions.size" :more-menu-actions="actions" @more-menu-action="{listener}" />
  </div>
  <slot v-if="loading" name="skeleton" />
  <slot v-else />
</template>
"#
    );
    let project = super::real_vue::create_project_case(
        "guarded-arrow-listener-11965",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ moreMenuActions: Map<string, boolean> }>();
defineEmits<{ (e: 'moreMenuAction', action: string): void }>();
</script>

<template><div /></template>
"#,
            ),
            ("src/Probe.vue", probe.as_str()),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    actual
}

#[test]
fn guarded_arrow_listener_with_callable_fallback_has_no_ts2774() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let actual = diagnostics("(e) => (handler || (() => {}))(e)");
    assert_eq!(actual, Some(Vec::new()));
}

#[test]
fn guarded_bare_listener_control_remains_clean() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    assert_eq!(diagnostics("handler"), Some(Vec::new()));
}

#[test]
fn guarded_arrow_without_fallback_keeps_possible_undefined_diagnostic() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let actual = diagnostics("(e) => handler(e)").expect("native diagnostics");
    assert_eq!(actual.len(), 1, "{actual:#?}");
    assert_eq!(actual[0].0, "src/Probe.vue");
    assert_eq!(actual[0].1, Some(2722));
    assert!(
        actual[0]
            .2
            .contains("Cannot invoke an object which is possibly 'undefined'."),
        "{actual:#?}"
    );
}
