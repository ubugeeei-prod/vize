//! Vue public props and adjacent checker paths for #3566, and every boundary
//! #3569 names: a correct named prop beside a missing sibling, a wrong named
//! prop beside a missing sibling, a complete usage, fallthrough attributes,
//! spreads, and one diagnostic per defect.
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
  <!-- Correct named values keep unbound required props active. -->
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
  <UnionChild kind="text" />
  <UnionChild kind="wrong" />
  <UnionChild kind="text" text="ok" />

  <!-- Multibyte text before the tag must not shift the reported column. -->
  <span title="🎉✅" /><Child :count="1" />

  <!-- A declared binding beside fallthrough attrs or a spread. -->
  <Child :count="1" class="edge" data-id="1" />
  <Child :count="1" v-bind="goodBag" />

  <!-- Spread order and synthetic singleton spreads preserve Vue last-wins. -->
  <Child v-bind="goodBag" :count="1" />
  <Child :count="1" v-bind="missingBag" label="middle" v-bind="goodBag" />
  <Child :[propName]="1" v-bind="goodBag" />
  <Child :count="1" :count="2" label="ok" />
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
            ("src/Parent.vue", Some(1117), "64:22".to_string()),
            ("src/Parent.vue", Some(2322), "19:11".to_string()),
            ("src/Parent.vue", Some(2322), "45:26".to_string()),
            ("src/Parent.vue", Some(2322), "50:15".to_string()),
            ("src/Parent.vue", Some(2345), "18:4".to_string()),
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
            ("src/Parent.vue", Some(2345), "49:4".to_string()),
            ("src/Parent.vue", Some(2345), "54:24".to_string()),
            ("src/Parent.vue", Some(2345), "57:4".to_string()),
        ],
        "exact vue-tsc positions: {snapshot:#?}"
    );

    // Every defect above is reported by exactly one check. The per-prop path and
    // the whole-props path both see a wrongly typed named prop, at the same
    // authored anchor and with the same message, so `dedup_diagnostics` collapses
    // the pair - a second row on any of these lines is that collapse breaking.
    let mut lines: Vec<_> = snapshot
        .iter()
        .map(|(_, _, message)| message.split(':').next().unwrap_or_default())
        .collect();
    let reported = lines.len();
    lines.sort_unstable();
    lines.dedup();
    assert_eq!(
        lines.len(),
        reported,
        "no usage may be reported twice: {snapshot:#?}"
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
        "the wrong declared prop must remain the sole diagnostic for its usage: {snapshot:#?}"
    );
    assert!(
        diagnostic_at(28)
            .2
            .contains("not assignable to type 'number'"),
        "the wrong spread prop must be a value mismatch: {snapshot:#?}"
    );
    for (line, expected) in [
        (18, "Property 'label' is missing"),
        (27, "Property 'label' is missing"),
        (39, "Property 'count' is missing"),
        (57, "Property 'label' is missing"),
    ] {
        assert!(
            diagnostic_at(line).2.contains(expected),
            "line {line} must report `{expected}`: {snapshot:#?}"
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
    assert!(
        diagnostic_at(49).2.contains("Property 'text' is missing"),
        "a valid discriminant must select the complete union arm: {snapshot:#?}"
    );
    assert!(
        diagnostic_at(50).2.contains("not assignable") && !diagnostic_at(50).2.contains("missing"),
        "an invalid discriminant must remain the sole value diagnostic: {snapshot:#?}"
    );
    assert!(
        diagnostic_at(64).1 == Some(1117),
        "a genuine duplicate named attribute must not be hidden by singleton spread codegen: {snapshot:#?}"
    );
    // Multibyte text before the tag must not shift the column: `🎉` is two UTF-16
    // code units and `✅` is one, so the `Child` tag name sits at UTF-16 column 24
    // (byte column 28, code-point column 23). Only the UTF-16 value is correct.
    assert!(
        diagnostic_at(54).2.starts_with("54:24:")
            && diagnostic_at(54).2.contains("Property 'label' is missing"),
        "a multibyte prefix must keep the reported column in UTF-16 units: {snapshot:#?}"
    );
}
