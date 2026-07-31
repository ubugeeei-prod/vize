//! Fallthrough-only component usages still check required props (#3566).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn fallthrough_only_usages_report_only_supported_missing_required_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "fallthrough-only-required-props",
        &[
            (
                "src/Required.vue",
                r#"<script setup lang="ts">
defineProps<{ count: number }>()
</script>
<template><span /></template>
"#,
            ),
            (
                "src/Optional.vue",
                r#"<script setup lang="ts">
defineProps<{ label?: string }>()
</script>
<template><span /></template>
"#,
            ),
            (
                "src/components.ts",
                r#"import type { DefineComponent } from 'vue'
export const External = {} as DefineComponent<{ count: number }>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import { Transition } from 'vue'
import Required from './Required.vue'
import Optional from './Optional.vue'
import { External } from './components'
</script>

<template>
  <Required class="static" />
  <Required :class="'bound'" />
  <Required data-id="1" />
  <Optional class="clean" />
  <External style="display: block" />
  <Transition class="clean" />
  <UnknownWidget aria-label="clean" />
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
    let mut positions: Vec<_> = snapshot
        .iter()
        .map(|(file, code, message)| {
            (
                file.as_str(),
                *code,
                message.split(':').take(2).collect::<Vec<_>>().join(":"),
            )
        })
        .collect();
    positions.sort_by_key(|(_, _, position)| {
        position
            .split(':')
            .next()
            .and_then(|line| line.parse::<u32>().ok())
            .unwrap_or_default()
    });

    assert_eq!(
        positions,
        [
            ("src/Parent.vue", Some(2345), "9:4".to_string()),
            ("src/Parent.vue", Some(2345), "10:4".to_string()),
            ("src/Parent.vue", Some(2345), "11:4".to_string()),
            ("src/Parent.vue", Some(2345), "13:4".to_string()),
        ],
        "exact fallthrough-only diagnostics: {snapshot:#?}"
    );
}

#[test]
fn an_explicit_class_prop_mismatch_is_reported_once() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "explicit-class-prop",
        &[
            (
                "src/ClassProp.vue",
                r#"<script setup lang="ts">
defineProps<{ class: number }>()
</script>
<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import ClassProp from './ClassProp.vue'
</script>

<template>
  <ClassProp class="bad" />
  <ClassProp :class="1" />
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
        snapshot.len(),
        1,
        "the mismatch must not duplicate: {snapshot:#?}"
    );
    assert_eq!(snapshot[0].0, "src/Parent.vue");
    assert_eq!(snapshot[0].1, Some(2322));
    assert!(
        snapshot[0].2.starts_with("6:14:"),
        "vue-tsc anchors a static prop mismatch at its name: {snapshot:#?}"
    );
}
