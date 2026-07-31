//! Empty component usages still check required props (#3527).

use super::super::{
    BatchTypeChecker, create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics,
};

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

/// Keep empty checks out of the template function's control-flow graph. A
/// large real-world page crossed TypeScript's TS2563 threshold when #3527
/// initially emitted one call statement per empty usage.
#[test]
fn many_empty_usages_do_not_exhaust_typescript_control_flow() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    // Mutation oracle: 600 guarded calls trigger TS2563 when emitted directly,
    // while unguarded calls remain below TypeScript's control-flow threshold.
    const USAGE_COUNT: usize = 600;
    let mut parent = std::string::String::from(
        r#"<script setup lang="ts">
import Required from './Required.vue'
const flag = true
</script>
<template>
"#,
    );
    for _ in 0..USAGE_COUNT {
        parent.push_str("  <Required v-if=\"flag\" />\n");
    }
    parent.push_str("</template>\n");

    let project_root = create_project_case(
        "many-empty-component-required-props",
        &[
            (
                "src/Required.vue",
                r#"<script setup lang="ts">
defineProps<{ count: number }>()
</script>
<template><span /></template>
"#,
            ),
            ("src/Parent.vue", parent.as_str()),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert!(
        snapshot.iter().all(|(_, code, _)| *code != Some(2563)),
        "empty usages must not make the template function too large: {snapshot:#?}"
    );
    assert_eq!(
        snapshot
            .iter()
            .filter(|(_, code, _)| *code == Some(2345))
            .count(),
        USAGE_COUNT,
        "every empty usage must retain its required-prop diagnostic"
    );
}

/// Closure scopes need the same isolation as root siblings. Inspect the emitted
/// callback directly because TypeScript's TS2563 threshold for a `v-for` body
/// is reached by the surrounding loop before empty-call isolation differs.
#[test]
fn v_for_empty_usages_are_isolated_inside_the_callback() {
    let parent = r#"<script setup lang="ts">
import Required from './Required.vue'
</script>
<template>
  <div v-for="item in [1]" :key="item">
    <Required />
    <Required />
  </div>
</template>
"#;

    let project_root = create_project_case(
        "v-for-empty-component-required-props-isolation",
        &[
            (
                "src/Required.vue",
                r#"<script setup lang="ts">
defineProps<{ count: number }>()
</script>
<template><span /></template>
"#,
            ),
            ("src/Parent.vue", parent),
        ],
    );

    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.scan_project().unwrap();
    let generated = checker
        .virtual_files()
        .into_iter()
        .find(|file| file.original_path.ends_with("src/Parent.vue"))
        .expect("Parent.vue virtual file")
        .content
        .as_str();
    let callback = generated
        .split_once("// Component props in v-for scope:")
        .expect("component props v-for section")
        .1
        .split_once("__vForList([1]).forEach(([item]) => {")
        .expect("v-for callback")
        .1;

    assert!(callback.contains("    void [\n"), "{callback}");
    assert_eq!(callback.matches("      () => {").count(), 2, "{callback}");
    let _ = std::fs::remove_dir_all(&project_root);
}
