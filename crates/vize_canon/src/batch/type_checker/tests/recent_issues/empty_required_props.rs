//! Empty component usages still check required props (#3527).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

/// Supported required-prop shapes report at the component name, while optional
/// runtime/mixin props, framework, and unresolved components stay clean.
#[test]
fn empty_usages_cover_supported_component_shapes_without_false_positives() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "empty-component-required-props",
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
                "src/RuntimeOptional.vue",
                r#"<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({ props: { label: String } })
</script>
<template><span /></template>
"#,
            ),
            (
                "src/Mixin.vue",
                r#"<script lang="ts">
import { defineComponent } from 'vue'
const base = defineComponent({ props: { label: String } })
export default defineComponent({ mixins: [base] })
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
import RuntimeOptional from './RuntimeOptional.vue'
import Mixin from './Mixin.vue'
import { External } from './components'
const propName = 'count' as string
</script>

<template>
  <Required />
  <Required :[propName]="1" />
  <Optional />
  <RuntimeOptional />
  <Mixin />
  <External />
  <Transition />
  <UnknownWidget />
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
    let positions: Vec<_> = snapshot
        .iter()
        .map(|(file, code, message)| {
            (
                file.as_str(),
                *code,
                message.split(':').take(2).collect::<Vec<_>>().join(":"),
            )
        })
        .collect();

    assert_eq!(
        positions,
        [
            ("src/Parent.vue", Some(2345), "12:4".to_string()),
            ("src/Parent.vue", Some(2345), "13:4".to_string()),
            ("src/Parent.vue", Some(2345), "17:4".to_string()),
        ],
        "exact required-prop diagnostics only: {snapshot:#?}"
    );
}
