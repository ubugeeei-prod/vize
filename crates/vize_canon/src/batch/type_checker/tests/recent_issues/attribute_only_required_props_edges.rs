//! Vue public props and adjacent checker paths for #3566.
//!
//! Oracle: `vue-tsc@3.3.4`, TypeScript `6.0.3`, Vue `3.6.0-beta.10`.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn required_props_survive_every_attribute_only_boundary() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "attribute-only-required-props-edges",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ count: number; label: string; tone?: 'info' | 'danger' }>()
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
                "src/UnionChild.vue",
                r#"<script setup lang="ts">
defineProps<
  | { kind: 'text'; text: string }
  | { kind: 'count'; count: number }
>()
</script>
<template><span /></template>
"#,
            ),
            (
                "src/EventChild.vue",
                r#"<script setup lang="ts">
defineProps<{ count: number }>()
defineEmits<{ save: [value: string] }>()
</script>
<template><span /></template>
"#,
            ),
            (
                "src/global.d.ts",
                r#"export {}

declare module '@vue/runtime-core' {
  interface ComponentCustomProps {
    globalToken?: string
  }
}
"#,
            ),
            (
                "src/components.ts",
                r#"import { defineComponent } from 'vue'

export const ExternalEventChild = defineComponent({
  props: {
    count: { type: Number, required: true },
    label: { type: String, required: true },
  },
  emits: {
    save: (_value: string) => true,
    cancel: (_value: string) => true,
    XML: (_value: string) => true,
  },
})

export const ExternalCallbackChild = defineComponent({
  props: {
    onCustom: { type: Function, required: true },
  },
})
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
import EventChild from './EventChild.vue'
import Optional from './Optional.vue'
import UnionChild from './UnionChild.vue'
import { ExternalCallbackChild, ExternalEventChild } from './components'

const propName = 'data-id'
const goodBag = { count: 1, label: 'ok' }
const missingBag = { count: 1 }
const badBag = { count: 'bad', label: 'ok' }
const onMounted = () => {}
const onSave = (_value: string) => {}
</script>

<template>
  <!-- #3569 boundary: its missing-label fix remains separate. -->
  <Child :count="1" />
  <Child :count="'bad'" />
  <Child :count="1" label="ok" />

  <!-- A union contract still requires one complete arm. -->
  <UnionChild data-id="union" />

  <!-- Fallthrough attrs beside spreads must not weaken the spread contract. -->
  <Child class="clean" v-bind="goodBag" />
  <Child style="color: red" v-bind="missingBag" />
  <Child class="bad-spread" v-bind="badBag" />

  <!-- Dynamic/reserved props are not declared component props. -->
  <Child :[propName]="1" />
  <Child ref="child" />
  <Child key="child" />

  <!-- Vue public/VNode/emits keys are not user-declared component props. -->
  <Child :ref_for="true" />
  <Child ref_key="child" />
  <Child :on-vnode-mounted="onMounted" />
  <EventChild :on-save="onSave" />
  <Child global-token="custom" />
  <ExternalEventChild :on-vnode-mounted="onMounted" />
  <ExternalEventChild :on-save="onSave" />
  <ExternalEventChild global-token="custom" />
  <ExternalEventChild :on-cancel="onSave" />
  <ExternalCallbackChild on-custom="bad" />
  <ExternalEventChild :on-XML="onSave" />

  <Optional class="clean" />
</template>
"#,
            ),
        ],
    );
    if !project_root.join("node_modules/vue/dist").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    let actual: Vec<_> = snapshot
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
        actual,
        [
            ("src/Parent.vue", Some(2322), "19:11".to_string()),
            ("src/Parent.vue", Some(2322), "45:26".to_string()),
            ("src/Parent.vue", Some(2345), "23:4".to_string()),
            ("src/Parent.vue", Some(2345), "27:4".to_string()),
            ("src/Parent.vue", Some(2345), "28:4".to_string()),
            ("src/Parent.vue", Some(2345), "31:4".to_string()),
            ("src/Parent.vue", Some(2345), "32:4".to_string()),
            ("src/Parent.vue", Some(2345), "33:4".to_string()),
            ("src/Parent.vue", Some(2345), "36:4".to_string()),
            ("src/Parent.vue", Some(2345), "37:4".to_string()),
            ("src/Parent.vue", Some(2345), "38:4".to_string()),
            ("src/Parent.vue", Some(2345), "39:4".to_string()),
            ("src/Parent.vue", Some(2345), "40:4".to_string()),
            ("src/Parent.vue", Some(2345), "41:4".to_string()),
            ("src/Parent.vue", Some(2345), "42:4".to_string()),
            ("src/Parent.vue", Some(2345), "43:4".to_string()),
            ("src/Parent.vue", Some(2345), "44:4".to_string()),
            ("src/Parent.vue", Some(2345), "46:4".to_string()),
        ],
        "exact vue-tsc positions, except the explicitly deferred #3569 row: {snapshot:#?}"
    );

    let diagnostic_at = |line: u32| {
        snapshot
            .iter()
            .find(|(_, _, message)| message.starts_with(&format!("{line}:")))
            .unwrap_or_else(|| panic!("missing diagnostic at line {line}: {snapshot:#?}"))
    };
    assert!(
        diagnostic_at(19)
            .2
            .contains("not assignable to type 'number'"),
        "the wrong declared prop must remain the sole #3569-boundary diagnostic: {snapshot:#?}"
    );
    assert!(
        diagnostic_at(28)
            .2
            .contains("not assignable to type 'number'"),
        "the wrong spread prop must be a value mismatch: {snapshot:#?}"
    );
    for line in [27, 39] {
        assert!(
            diagnostic_at(line)
                .2
                .contains("Property 'label' is missing")
                || diagnostic_at(line)
                    .2
                    .contains("Property 'count' is missing"),
            "line {line} must report its missing required prop: {snapshot:#?}"
        );
    }
    for line in [23, 31, 32, 33, 36, 37, 38, 40, 41, 42, 43, 44, 46] {
        assert!(
            diagnostic_at(line).2.contains("missing"),
            "line {line} must be a missing-required-prop diagnostic: {snapshot:#?}"
        );
    }
    assert!(
        diagnostic_at(45).2.contains("not assignable"),
        "a declared on* callback must stay on the ordinary prop path: {snapshot:#?}"
    );
}
